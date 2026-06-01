import { TestBed, ComponentFixture } from '@angular/core/testing';
import { DashboardComponent } from './dashboard.component';
import { AuthService } from '../../services/auth.service';
import { Router } from '@angular/router';
import { signal } from '@angular/core';

describe('DashboardComponent', () => {
  let component: DashboardComponent;
  let fixture: ComponentFixture<DashboardComponent>;
  let mockAuthService: any;
  let mockRouter: any;
  let mockUserSignal: any;

  beforeEach(async () => {
    mockUserSignal = signal({
      id: 'user_999',
      fullName: 'Jane Doe',
      primaryEmailAddress: 'jane@example.com',
      imageUrl: 'http://avatar.com/jane.jpg'
    });

    mockAuthService = {
      user: mockUserSignal,
      signOut: vi.fn().mockResolvedValue(undefined)
    };

    mockRouter = {
      navigate: vi.fn()
    };

    await TestBed.configureTestingModule({
      imports: [DashboardComponent],
      providers: [
        { provide: AuthService, useValue: mockAuthService },
        { provide: Router, useValue: mockRouter }
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(DashboardComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render user profile details', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.welcome-title')?.textContent).toContain('Welcome back, Jane Doe!');
    expect(compiled.querySelector('.profile-details-grid')?.textContent).toContain('jane@example.com');
    expect(compiled.querySelector('.mono-text')?.textContent).toContain('user_999');
    
    const img = compiled.querySelector('.user-avatar') as HTMLImageElement;
    expect(img.src).toBe('http://avatar.com/jane.jpg');
  });

  it('should trigger auth signOut and routing to root on logout click', async () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const button = compiled.querySelector('.btn-logout') as HTMLButtonElement;
    
    button.click();
    await fixture.whenStable();

    expect(mockAuthService.signOut).toHaveBeenCalled();
    expect(mockRouter.navigate).toHaveBeenCalledWith(['/']);
  });
});
