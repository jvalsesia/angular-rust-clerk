import { Injectable, signal, computed, inject, effect } from '@angular/core';
import { ClerkService } from './clerk.service';

export interface UserProfile {
  id: string;
  fullName: string | null;
  primaryEmailAddress: string | null;
  imageUrl: string;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private clerkService = inject(ClerkService);
  
  private _user = signal<UserProfile | null>(null);
  private _isLoaded = signal(false);
  
  public user = this._user.asReadonly();
  public isLoaded = this._isLoaded.asReadonly();
  public isAuthenticated = computed(() => this._user() !== null);
  
  constructor() {
    effect(() => {
      if (this.clerkService.isLoaded()) {
        const clerk = this.clerkService.getClerk();
        if (clerk) {
          // Sync initial state
          this.updateState(clerk.user);
          
          // Subscribe to Clerk SDK session/user transitions
          clerk.addListener((state: { user: any }) => {
            this.updateState(state.user);
          });
          
          this._isLoaded.set(true);
        }
      }
    });
  }
  
  private updateState(clerkUser: any) {
    if (clerkUser) {
      this._user.set({
        id: clerkUser.id,
        fullName: clerkUser.fullName,
        primaryEmailAddress: clerkUser.primaryEmailAddress?.emailAddress ?? null,
        imageUrl: clerkUser.imageUrl
      });
    } else {
      this._user.set(null);
    }
  }

  /**
   * Fetches the current session token (JWT) from Clerk.
   */
  public async getToken(): Promise<string | null> {
    const clerk = this.clerkService.getClerk();
    if (clerk && clerk.session) {
      return await clerk.session.getToken();
    }
    return null;
  }

  /**
   * Logs out the current user session.
   */
  public async signOut(): Promise<void> {
    const clerk = this.clerkService.getClerk();
    if (clerk) {
      await clerk.signOut();
    }
  }
}
