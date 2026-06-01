import { TestBed } from '@angular/core/testing';
import { LoginComponent } from './login.component';
import { ClerkService } from '../../services/clerk.service';
import { signal } from '@angular/core';
import { provideRouter } from '@angular/router';

describe('LoginComponent', () => {
  let mockClerkService: any;

  beforeEach(async () => {
    mockClerkService = {
      isLoaded: signal(false),
      mountSignIn: () => {}
    };

    // Spy on mountSignIn
    mockClerkService.mountSignIn = vi.fn();

    await TestBed.configureTestingModule({
      imports: [LoginComponent],
      providers: [
        provideRouter([]),
        { provide: ClerkService, useValue: mockClerkService }
      ]
    }).compileComponents();
  });

  it('should create', () => {
    const fixture = TestBed.createComponent(LoginComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
  });

  it('should display progress spinner while loading', () => {
    mockClerkService.isLoaded.set(false);
    const fixture = TestBed.createComponent(LoginComponent);
    fixture.detectChanges();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('mat-progress-spinner')).toBeTruthy();
    expect(compiled.textContent).toContain('Loading Authentication Services');
  });

  it('should hide spinner and call mount when loaded', () => {
    mockClerkService.isLoaded.set(true);
    const fixture = TestBed.createComponent(LoginComponent);
    fixture.detectChanges();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('mat-progress-spinner')).toBeFalsy();
    expect(mockClerkService.mountSignIn).toHaveBeenCalled();
  });
});
