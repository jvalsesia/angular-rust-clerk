import { Injectable, signal } from '@angular/core';

declare global {
  interface Window {
    Clerk?: any;
  }
}

@Injectable({
  providedIn: 'root'
})
export class ClerkService {
  private clerkInstance: any = null;
  public isLoaded = signal(false);
  
  // Use a default sandbox key, check window or localStorage for overrides if needed
  private publishableKey = (typeof window !== 'undefined' && (window as any).CLERK_PUBLISHABLE_KEY) || 
                           (typeof localStorage !== 'undefined' && localStorage.getItem('CLERK_PUBLISHABLE_KEY')) || 
                           'pk_test_a2luZC12ZXJ2ZXQtMzguY2xlcmsuYWNjb3VudHMuZGV2JA';

  constructor() {
    this.loadClerkSDK();
  }

  private loadClerkSDK() {
    if (this.isLoaded()) return;

    if (window.Clerk && window.Clerk.isReady) {
      this.clerkInstance = window.Clerk;
      this.isLoaded.set(true);
      return;
    }

    const script = document.createElement('script');
    script.src = 'https://unpkg.com/@clerk/clerk-js@5/dist/clerk.browser.js';
    script.async = true;
    script.crossOrigin = 'anonymous';
    script.setAttribute('data-clerk-publishable-key', this.publishableKey);
    script.onload = async () => {
      const clerkGlobal = (window as any).Clerk;
      if (clerkGlobal) {
        try {
          let instance = clerkGlobal;
          if (typeof clerkGlobal === 'function') {
            instance = new clerkGlobal(this.publishableKey);
          }
          await instance.load();
          this.clerkInstance = instance;
          this.isLoaded.set(true);
        } catch (err) {
          console.error('Failed to initialize Clerk instance:', err);
        }
      }
    };
    script.onerror = (err) => {
      console.error('Failed to load Clerk script tag:', err);
    };

    document.head.appendChild(script);
  }

  public getClerk() {
    return this.clerkInstance;
  }

  public mountSignIn(target: HTMLElement) {
    if (this.clerkInstance) {
      this.clerkInstance.mountSignIn(target, {
        afterSignInUrl: '/dashboard',
        afterSignUpUrl: '/dashboard'
      });
    }
  }

  public mountSignUp(target: HTMLElement) {
    if (this.clerkInstance) {
      this.clerkInstance.mountSignUp(target, {
        afterSignInUrl: '/dashboard',
        afterSignUpUrl: '/dashboard'
      });
    }
  }
}
