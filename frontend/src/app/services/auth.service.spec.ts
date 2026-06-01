import { TestBed } from '@angular/core/testing';
import { AuthService } from './auth.service';
import { ClerkService } from './clerk.service';
import { signal } from '@angular/core';

describe('AuthService', () => {
  let service: AuthService;
  let mockClerkService: any;
  let isClerkLoadedSignal: any;
  let mockClerkInstance: any;

  beforeEach(() => {
    isClerkLoadedSignal = signal(false);
    mockClerkInstance = {
      user: null,
      addListener: vi.fn(),
      session: {
        getToken: vi.fn().mockResolvedValue('test-jwt')
      },
      signOut: vi.fn().mockResolvedValue(undefined)
    };

    mockClerkService = {
      isLoaded: isClerkLoadedSignal,
      getClerk: () => mockClerkInstance
    };

    TestBed.configureTestingModule({
      providers: [
        AuthService,
        { provide: ClerkService, useValue: mockClerkService }
      ]
    });
    service = TestBed.inject(AuthService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  it('should report isAuthenticated as false initially when not loaded', () => {
    expect(service.isLoaded()).toBe(false);
    expect(service.isAuthenticated()).toBe(false);
    expect(service.user()).toBeNull();
  });

  it('should update state when clerk resolves user', () => {
    // Mock user profile details
    mockClerkInstance.user = {
      id: 'user_123',
      fullName: 'John Doe',
      primaryEmailAddress: { emailAddress: 'john@example.com' },
      imageUrl: 'http://image.com'
    };

    // Trigger effect
    isClerkLoadedSignal.set(true);
    TestBed.flushEffects();

    expect(service.isLoaded()).toBe(true);
    expect(service.isAuthenticated()).toBe(true);
    expect(service.user()).toEqual({
      id: 'user_123',
      fullName: 'John Doe',
      primaryEmailAddress: 'john@example.com',
      imageUrl: 'http://image.com'
    });
    expect(mockClerkInstance.addListener).toHaveBeenCalled();
  });
});
