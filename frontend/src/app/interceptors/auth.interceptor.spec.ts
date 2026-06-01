import { TestBed } from '@angular/core/testing';
import { HttpClient, provideHttpClient, withInterceptors } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { authInterceptor } from './auth.interceptor';
import { AuthService } from '../services/auth.service';

describe('authInterceptor', () => {
  let httpMock: HttpTestingController;
  let httpClient: HttpClient;
  let mockAuthService: any;

  beforeEach(() => {
    mockAuthService = {
      getToken: vi.fn().mockResolvedValue('fake-clerk-jwt')
    };

    TestBed.configureTestingModule({
      providers: [
        provideHttpClient(withInterceptors([authInterceptor])),
        provideHttpClientTesting(),
        { provide: AuthService, useValue: mockAuthService }
      ]
    });

    httpMock = TestBed.inject(HttpTestingController);
    httpClient = TestBed.inject(HttpClient);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('should attach Authorization Bearer header for /api/ requests', async () => {
    httpClient.get('/api/health').subscribe();

    // Flush macro task queue to resolve the getToken promise
    await new Promise(resolve => setTimeout(resolve, 0));

    const req = httpMock.expectOne('/api/health');
    expect(req.request.headers.has('Authorization')).toBe(true);
    expect(req.request.headers.get('Authorization')).toBe('Bearer fake-clerk-jwt');
    req.flush({ status: 'ok' });
  });

  it('should NOT attach Authorization header for external requests', async () => {
    httpClient.get('https://api.clerk.com/v1/user').subscribe();

    // Flush macro task queue to resolve the getToken promise
    await new Promise(resolve => setTimeout(resolve, 0));

    const req = httpMock.expectOne('https://api.clerk.com/v1/user');
    expect(req.request.headers.has('Authorization')).toBe(false);
    req.flush({});
  });

  it('should send request unmodified if token is null', async () => {
    mockAuthService.getToken.mockResolvedValue(null);
    httpClient.get('/api/health').subscribe();

    await new Promise(resolve => setTimeout(resolve, 0));

    const req = httpMock.expectOne('/api/health');
    expect(req.request.headers.has('Authorization')).toBe(false);
    req.flush({});
  });
});
