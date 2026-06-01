import { TestBed } from '@angular/core/testing';
import { RegisterComponent } from './register.component';
import { ClerkService } from '../../services/clerk.service';
import { signal } from '@angular/core';
import { provideRouter } from '@angular/router';

describe('RegisterComponent', () => {
  let mockClerkService: any;

  beforeEach(async () => {
    mockClerkService = {
      isLoaded: signal(false),
      mountSignUp: () => {}
    };

    // Spy on mountSignUp
    mockClerkService.mountSignUp = vi.fn();

    await TestBed.configureTestingModule({
      imports: [RegisterComponent],
      providers: [
        provideRouter([]),
        { provide: ClerkService, useValue: mockClerkService }
      ]
    }).compileComponents();
  });

  it('should create', () => {
    const fixture = TestBed.createComponent(RegisterComponent);
    const component = fixture.componentInstance;
    expect(component).toBeTruthy();
  });

  it('should display progress spinner while loading', () => {
    mockClerkService.isLoaded.set(false);
    const fixture = TestBed.createComponent(RegisterComponent);
    fixture.detectChanges();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('mat-progress-spinner')).toBeTruthy();
    expect(compiled.textContent).toContain('Loading Authentication Services');
  });

  it('should hide spinner and call mount when loaded', () => {
    mockClerkService.isLoaded.set(true);
    const fixture = TestBed.createComponent(RegisterComponent);
    fixture.detectChanges();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('mat-progress-spinner')).toBeFalsy();
    expect(mockClerkService.mountSignUp).toHaveBeenCalled();
  });
});
