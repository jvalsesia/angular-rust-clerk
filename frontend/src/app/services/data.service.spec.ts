import { TestBed } from '@angular/core/testing';
import { DataService, ProtectedData } from './data.service';
import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';

describe('DataService', () => {
  let service: DataService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        DataService,
        provideHttpClient(),
        provideHttpClientTesting()
      ]
    });
    service = TestBed.inject(DataService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  it('should fetch protected data from /api/protected via GET', () => {
    const dummyData: ProtectedData = {
      message: 'Access granted',
      user_id: 'user_123',
      timestamp: 1715000000
    };

    service.getProtectedData().subscribe(data => {
      expect(data).toEqual(dummyData);
    });

    const req = httpMock.expectOne('/api/protected');
    expect(req.request.method).toBe('GET');
    req.flush(dummyData);
  });
});
