import { TestBed } from '@angular/core/testing';
import { ChatService, ChatEvent } from './chat.service';
import { AuthService } from './auth.service';

describe('ChatService', () => {
  let service: ChatService;
  let mockAuthService: any;

  beforeEach(() => {
    mockAuthService = {
      getToken: vi.fn().mockResolvedValue('mock-token')
    };

    TestBed.configureTestingModule({
      providers: [
        ChatService,
        { provide: AuthService, useValue: mockAuthService }
      ]
    });
    service = TestBed.inject(ChatService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  it('should stream chat events correctly and parse structured SSE lines', async () => {
    const encoder = new TextEncoder();
    const chunks = [
      'event: session_created\ndata: {"session_id": "session-123"}\n\n',
      'event: context\ndata: [{"role":"user","content":"hello"}]\n\n',
      'event: token\ndata: {"text": "Hi"}\n\n',
      'event: done\ndata: {}\n\n'
    ];

    let chunkIndex = 0;
    const mockReader = {
      read: vi.fn().mockImplementation(() => {
        if (chunkIndex < chunks.length) {
          const value = encoder.encode(chunks[chunkIndex]);
          chunkIndex++;
          return Promise.resolve({ value, done: false });
        }
        return Promise.resolve({ value: undefined, done: true });
      })
    };

    const mockResponse = {
      ok: true,
      status: 200,
      statusText: 'OK',
      body: {
        getReader: () => mockReader
      }
    };

    const fetchSpy = vi.stubGlobal('fetch', vi.fn().mockResolvedValue(mockResponse));

    const events: ChatEvent[] = [];
    await new Promise<void>((resolve, reject) => {
      service.streamChat('hello', 'gpt-4o-mini', null).subscribe({
        next: (evt) => {
          events.push(evt);
        },
        error: (err) => reject(err),
        complete: () => resolve()
      });
    });

    expect(events.length).toBe(4);
    expect(events[0]).toEqual({ type: 'session_created', sessionId: 'session-123' });
    expect(events[1]).toEqual({ type: 'context', messages: [{ role: 'user', content: 'hello' }] });
    expect(events[2]).toEqual({ type: 'token', text: 'Hi' });
    expect(events[3]).toEqual({ type: 'done' });

    vi.unstubAllGlobals();
  });

  it('should push error event if fetch fails', async () => {
    const mockResponse = {
      ok: false,
      status: 500,
      statusText: 'Internal Server Error'
    };

    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(mockResponse));

    const events: ChatEvent[] = [];
    await new Promise<void>((resolve) => {
      service.streamChat('hello', 'gpt-4o-mini', null).subscribe({
        next: (evt) => {
          events.push(evt);
        },
        complete: () => resolve()
      });
    });

    expect(events.length).toBe(1);
    expect(events[0].type).toBe('error');
    if (events[0].type === 'error') {
      expect(events[0].error).toContain('HTTP error 500');
    }

    vi.unstubAllGlobals();
  });
});
