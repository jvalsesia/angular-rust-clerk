import { TestBed, ComponentFixture } from '@angular/core/testing';
import { ChatComponent } from './chat.component';
import { ChatService, ChatEvent } from '../../services/chat.service';
import { AuthService } from '../../services/auth.service';
import { Router, provideRouter } from '@angular/router';
import { signal } from '@angular/core';
import { Subject } from 'rxjs';

describe('ChatComponent', () => {
  let component: ChatComponent;
  let fixture: ComponentFixture<ChatComponent>;
  let mockChatService: any;
  let mockAuthService: any;
  let router: Router;
  let mockUserSignal: any;
  let chatStream$: Subject<ChatEvent>;

  beforeEach(async () => {
    mockUserSignal = signal({
      id: 'user_chat_123',
      fullName: 'Chat Tester',
      primaryEmailAddress: 'chat@test.com',
      imageUrl: 'http://avatar.com/chat.jpg'
    });

    mockAuthService = {
      user: mockUserSignal,
      signOut: vi.fn().mockResolvedValue(undefined)
    };

    chatStream$ = new Subject<ChatEvent>();
    mockChatService = {
      streamChat: vi.fn().mockReturnValue(chatStream$)
    };

    await TestBed.configureTestingModule({
      imports: [ChatComponent],
      providers: [
        provideRouter([]),
        { provide: AuthService, useValue: mockAuthService },
        { provide: ChatService, useValue: mockChatService }
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(ChatComponent);
    component = fixture.componentInstance;
    router = TestBed.inject(Router);
    vi.spyOn(router, 'navigate').mockImplementation(() => Promise.resolve(true));
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should update selected model when model changes', () => {
    const selectEl = { target: { value: 'gemini-2.5-flash' } } as unknown as Event;
    component.onModelChange(selectEl);
    expect(component.selectedModel()).toBe('gemini-2.5-flash');
  });

  it('should send prompt and stream tokens progressively', () => {
    // 1. Submit prompt
    component.onPromptChange('What is vector similarity?');
    component.sendMessage();

    // Verify loading state
    expect(component.isStreaming()).toBe(true);
    expect(component.chatHistory().length).toBe(2);
    expect(component.chatHistory()[0]).toEqual({ role: 'user', content: 'What is vector similarity?' });
    expect(component.chatHistory()[1]).toEqual({
      role: 'assistant',
      content: '',
      modelName: 'gpt-4o-mini',
      embeddingModel: 'text-embedding-3-small'
    });
    expect(mockChatService.streamChat).toHaveBeenCalledWith('What is vector similarity?', 'gpt-4o-mini', null);

    // 2. Emit context event
    chatStream$.next({
      type: 'context',
      messages: [{ role: 'user', content: 'hi' }]
    });
    expect(component.relevantContext().length).toBe(1);
    expect(component.relevantContext()[0]).toEqual({ role: 'user', content: 'hi' });

    // 3. Emit tokens
    chatStream$.next({ type: 'token', text: 'Vector ' });
    chatStream$.next({ type: 'token', text: 'search' });

    expect(component.chatHistory()[1].content).toBe('Vector search');

    // 4. Emit done
    chatStream$.next({ type: 'done' });
    expect(component.isStreaming()).toBe(false);
  });

  it('should toggle sidebar drawer state', () => {
    expect(component.isDrawerOpen()).toBe(true);
    component.toggleDrawer();
    expect(component.isDrawerOpen()).toBe(false);
    component.toggleDrawer();
    expect(component.isDrawerOpen()).toBe(true);
  });

  it('should cleanly sign out and route to landing page', async () => {
    await component.onSignOut();
    expect(mockAuthService.signOut).toHaveBeenCalled();
    expect(router.navigate).toHaveBeenCalledWith(['/']);
  });
});
