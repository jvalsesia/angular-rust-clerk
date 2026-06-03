import { TestBed, ComponentFixture } from '@angular/core/testing';
import { DashboardComponent } from './dashboard.component';
import { AuthService } from '../../services/auth.service';
import { DataService } from '../../services/data.service';
import { Router, provideRouter } from '@angular/router';
import { signal } from '@angular/core';
import { of } from 'rxjs';

describe('DashboardComponent', () => {
  let component: DashboardComponent;
  let fixture: ComponentFixture<DashboardComponent>;
  let mockAuthService: any;
  let mockDataService: any;
  let router: Router;
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

    mockDataService = {
      getProtectedData: vi.fn().mockReturnValue(of({
        message: 'Mock Secure Message',
        user_id: 'user_999',
        timestamp: 1715000000
      }))
    };

    await TestBed.configureTestingModule({
      imports: [DashboardComponent],
      providers: [
        provideRouter([]),
        { provide: AuthService, useValue: mockAuthService },
        { provide: DataService, useValue: mockDataService }
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(DashboardComponent);
    component = fixture.componentInstance;
    router = TestBed.inject(Router);
    vi.spyOn(router, 'navigate').mockImplementation(() => Promise.resolve(true));
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

  it('should render active API integration card data', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.api-card-title')?.textContent).toContain('Secure Server Validation');
    expect(compiled.querySelector('.api-card-message')?.textContent).toContain('Mock Secure Message');
    expect(compiled.querySelector('.api-meta-details')?.textContent).toContain('user_999');
  });

  it('should trigger auth signOut and routing to root on logout click', async () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const button = compiled.querySelector('.btn-logout') as HTMLButtonElement;
    
    button.click();
    await fixture.whenStable();

    expect(mockAuthService.signOut).toHaveBeenCalled();
    expect(router.navigate).toHaveBeenCalledWith(['/']);
  });
});
