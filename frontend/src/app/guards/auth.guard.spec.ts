import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import { authGuard } from './auth.guard';
import { AuthService } from '../services/auth.service';
import { signal } from '@angular/core';

describe('authGuard', () => {
  let mockAuthService: any;
  let mockRouter: any;
  let isLoadedSignal: any;
  let isAuthenticatedSignal: any;

  beforeEach(() => {
    isLoadedSignal = signal(false);
    isAuthenticatedSignal = signal(false);

    mockAuthService = {
      isLoaded: isLoadedSignal,
      isAuthenticated: isAuthenticatedSignal
    };

    mockRouter = {
      navigate: vi.fn()
    };

    TestBed.configureTestingModule({
      providers: [
        { provide: AuthService, useValue: mockAuthService },
        { provide: Router, useValue: mockRouter }
      ]
    });
  });

  const runGuard = async (): Promise<boolean> => {
    const guardResult = TestBed.runInInjectionContext(() => 
      authGuard({} as any, {} as any)
    );
    return await (guardResult as Promise<boolean>);
  };

  it('should allow access if authenticated and loaded', async () => {
    isLoadedSignal.set(true);
    isAuthenticatedSignal.set(true);

    const result = await runGuard();
    expect(result).toBe(true);
    expect(mockRouter.navigate).not.toHaveBeenCalled();
  });

  it('should redirect and deny access if not authenticated', async () => {
    isLoadedSignal.set(true);
    isAuthenticatedSignal.set(false);

    const result = await runGuard();
    expect(result).toBe(false);
    expect(mockRouter.navigate).toHaveBeenCalledWith(['/login']);
  });
});
