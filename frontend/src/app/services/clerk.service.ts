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
  private publishableKey = 'pk_test_Z2VudGxlLW9waGFwaC05OC5jbGVyay5hY2NvdW50cy5kZXYk';

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
    script.onload = async () => {
      const ClerkClass = (window as any).Clerk;
      if (ClerkClass) {
        try {
          const clerk = new ClerkClass(this.publishableKey);
          await clerk.load();
          this.clerkInstance = clerk;
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
