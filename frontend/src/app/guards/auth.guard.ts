import { CanActivateFn, Router } from '@angular/router';
import { inject } from '@angular/core';
import { AuthService } from '../services/auth.service';
import { toObservable } from '@angular/core/rxjs-interop';
import { filter, firstValueFrom, timeout, catchError, of } from 'rxjs';

export const authGuard: CanActivateFn = async (route, state) => {
  const authService = inject(AuthService);
  const router = inject(Router);

  // If already loaded, evaluate immediately
  if (authService.isLoaded()) {
    if (authService.isAuthenticated()) {
      return true;
    }
    router.navigate(['/login']);
    return false;
  }

  // Otherwise, wait for Clerk to resolve session state with a 5-second timeout
  try {
    const isLoaded = await firstValueFrom(
      toObservable(authService.isLoaded).pipe(
        filter((loaded): loaded is boolean => loaded),
        timeout(5000),
        catchError(() => of(false))
      )
    );

    if (isLoaded && authService.isAuthenticated()) {
      return true;
    }
  } catch (e) {
    // Falls through to unauthorized redirect on failure/timeout
  }

  router.navigate(['/login']);
  return false;
};
