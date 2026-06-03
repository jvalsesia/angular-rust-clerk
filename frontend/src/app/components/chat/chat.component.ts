import { Component, inject, signal, OnDestroy, ChangeDetectionStrategy, ViewChild, ElementRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';
import { Subscription } from 'rxjs';
import { AuthService } from '../../services/auth.service';
import { ChatService, ContextMessage } from '../../services/chat.service';

export interface Message {
  role: 'user' | 'assistant';
  content: string;
  modelName?: string;
  embeddingModel?: string;
}

@Component({
  selector: 'app-chat',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink],
  templateUrl: './chat.component.html',
  styleUrls: ['./chat.component.css'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ChatComponent implements OnDestroy {
  private chatService = inject(ChatService);
  private authService = inject(AuthService);
  private router = inject(Router);

  public user = this.authService.user;

  @ViewChild('chatHistoryLog') private chatHistoryLog?: ElementRef<HTMLDivElement>;

  private scrollToBottom(force: boolean = false) {
    if (this.chatHistoryLog) {
      const element = this.chatHistoryLog.nativeElement;
      const threshold = 150; // px
      const isNearBottom = element.scrollHeight - element.scrollTop - element.clientHeight < threshold;
      if (force || isNearBottom) {
        element.scrollTop = element.scrollHeight;
      }
    }
  }

  private scheduleScrollToBottom(force: boolean = false) {
    setTimeout(() => this.scrollToBottom(force), 0);
  }
  public chatHistory = signal<Message[]>([]);
  public selectedModel = signal<string>('gpt-4o-mini');
  public isStreaming = signal<boolean>(false);
  public relevantContext = signal<ContextMessage[]>([]);
  public currentSessionId = signal<string | null>(null);
  public errorMessage = signal<string | null>(null);
  public inputPrompt = signal<string>('');
  public isDrawerOpen = signal<boolean>(true);

  private activeSubscription?: Subscription;

  public availableModels = [
    { value: 'gpt-4o-mini', label: 'GPT-4o Mini (Default)' },
    { value: 'gpt-4o', label: 'GPT-4o' },
    { value: 'o1-preview', label: 'o1 Preview' },
    { value: 'gemini-2.5-flash', label: 'Gemini 2.5 Flash' },
    { value: 'gemini-1.5-flash', label: 'Gemini 1.5 Flash' },
    { value: 'gemini-1.5-pro', label: 'Gemini 1.5 Pro' }
  ];

  public onModelChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    this.selectedModel.set(select.value);
  }

  public onPromptChange(value: string) {
    this.inputPrompt.set(value);
  }

  public sendMessage() {
    const promptText = this.inputPrompt();
    if (!promptText.trim() || this.isStreaming()) return;

    // Reset current states
    this.errorMessage.set(null);
    this.isStreaming.set(true);

    // Append user message
    const userMsg: Message = { role: 'user', content: promptText };
    this.chatHistory.update((history) => [...history, userMsg]);
    this.inputPrompt.set('');
    this.scheduleScrollToBottom(true);

    // Append initial empty assistant message to write streamed tokens into
    const assistantMsgIndex = this.chatHistory().length;
    const model = this.selectedModel();
    const embeddingModel = model.startsWith('gpt') || model.startsWith('o1')
      ? 'text-embedding-3-small'
      : 'text-embedding-005';

    this.chatHistory.update((history) => [
      ...history,
      {
        role: 'assistant',
        content: '',
        modelName: model,
        embeddingModel: embeddingModel
      }
    ]);
    this.scheduleScrollToBottom(true);

    let assistantResponse = '';

    // Unsubscribe from any active streams
    if (this.activeSubscription) {
      this.activeSubscription.unsubscribe();
    }

    // Call service to start SSE streaming
    this.activeSubscription = this.chatService
      .streamChat(promptText, this.selectedModel(), this.currentSessionId())
      .subscribe({
        next: (evt) => {
          if (evt.type === 'session_created') {
            this.currentSessionId.set(evt.sessionId);
          } else if (evt.type === 'context') {
            this.relevantContext.set(evt.messages);
          } else if (evt.type === 'token') {
            assistantResponse += evt.text;
            this.chatHistory.update((history) => {
              const updated = [...history];
              if (updated[assistantMsgIndex]) {
                updated[assistantMsgIndex] = {
                  ...updated[assistantMsgIndex],
                  content: assistantResponse
                };
              }
              return updated;
            });
            this.scheduleScrollToBottom(false);
          } else if (evt.type === 'error') {
            this.errorMessage.set(evt.error);
            this.isStreaming.set(false);
          } else if (evt.type === 'done') {
            this.isStreaming.set(false);
          }
        },
        error: (err) => {
          this.errorMessage.set(err.message || 'Stream connection closed unexpectedly');
          this.isStreaming.set(false);
        },
        complete: () => {
          this.isStreaming.set(false);
        }
      });
  }

  public toggleDrawer() {
    this.isDrawerOpen.update((open) => !open);
  }

  public async onSignOut() {
    if (this.activeSubscription) {
      this.activeSubscription.unsubscribe();
    }
    await this.authService.signOut();
    this.router.navigate(['/']);
  }

  public ngOnDestroy() {
    if (this.activeSubscription) {
      this.activeSubscription.unsubscribe();
    }
  }
}
