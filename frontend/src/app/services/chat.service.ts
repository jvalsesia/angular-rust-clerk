import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { AuthService } from './auth.service';

export interface ContextMessage {
  role: string;
  content: string;
}

export interface SessionCreatedEvent {
  type: 'session_created';
  sessionId: string;
}

export interface ContextEvent {
  type: 'context';
  messages: ContextMessage[];
}

export interface TokenEvent {
  type: 'token';
  text: string;
}

export interface ErrorEvent {
  type: 'error';
  error: string;
}

export interface DoneEvent {
  type: 'done';
}

export type ChatEvent = SessionCreatedEvent | ContextEvent | TokenEvent | ErrorEvent | DoneEvent;

@Injectable({
  providedIn: 'root'
})
export class ChatService {
  private authService = inject(AuthService);

  /**
   * Submits a prompt to /api/chat and streams SSE events progressively.
   * Cancels the stream via AbortController if unsubscribed.
   */
  public streamChat(
    prompt: string,
    model: string,
    sessionId: string | null
  ): Observable<ChatEvent> {
    return new Observable<ChatEvent>((subscriber) => {
      let aborted = false;
      const abortController = new AbortController();

      const runStream = async () => {
        try {
          const token = await this.authService.getToken();
          if (aborted) return;

          const response = await fetch('/api/chat', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              ...(token ? { 'Authorization': `Bearer ${token}` } : {})
            },
            body: JSON.stringify({
              session_id: sessionId,
              prompt,
              model
            }),
            signal: abortController.signal
          });

          if (!response.ok) {
            subscriber.next({
              type: 'error',
              error: `HTTP error ${response.status}: ${response.statusText}`
            });
            subscriber.complete();
            return;
          }

          const reader = response.body?.getReader();
          if (!reader) {
            subscriber.next({
              type: 'error',
              error: 'Response body stream is not readable'
            });
            subscriber.complete();
            return;
          }

          const decoder = new TextDecoder();
          let buffer = '';
          let currentEvent: string | null = null;

          while (!aborted) {
            const { value, done } = await reader.read();
            if (done) break;

            buffer += decoder.decode(value, { stream: true });
            const lines = buffer.split('\n');
            buffer = lines.pop() || '';

            for (const line of lines) {
              const trimmed = line.trim();
              if (trimmed.startsWith('event:')) {
                currentEvent = trimmed.substring(6).trim();
              } else if (trimmed.startsWith('data:')) {
                const dataStr = trimmed.substring(5).trim();
                if (currentEvent) {
                  if (currentEvent === 'done') {
                    subscriber.next({ type: 'done' });
                  } else {
                    try {
                      const parsed = JSON.parse(dataStr);
                      if (currentEvent === 'session_created') {
                        subscriber.next({
                          type: 'session_created',
                          sessionId: parsed.session_id
                        });
                      } else if (currentEvent === 'context') {
                        subscriber.next({
                          type: 'context',
                          messages: parsed
                        });
                      } else if (currentEvent === 'token') {
                        subscriber.next({
                          type: 'token',
                          text: parsed.text
                        });
                      } else if (currentEvent === 'error') {
                        subscriber.next({
                          type: 'error',
                          error: parsed.error
                        });
                      }
                    } catch (e) {
                      console.error('Failed to parse SSE data:', e, dataStr);
                    }
                  }
                  currentEvent = null;
                }
              }
            }
          }

          // Complete decoding buffer and finalize
          subscriber.complete();
        } catch (err: any) {
          if (err.name === 'AbortError') {
            // Cancelled cleanly
            return;
          }
          subscriber.next({
            type: 'error',
            error: err.message || 'Stream processing failed'
          });
          subscriber.complete();
        }
      };

      runStream();

      return () => {
        aborted = true;
        abortController.abort();
      };
    });
  }
}
