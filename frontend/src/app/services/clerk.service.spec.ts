import { TestBed } from '@angular/core/testing';
import { ClerkService } from './clerk.service';

describe('ClerkService', () => {
  let service: ClerkService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ClerkService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  it('should append the clerk script to the document body or head', () => {
    const scripts = Array.from(document.querySelectorAll('script'));
    const hasClerkScript = scripts.some(s => s.src.includes('unpkg.com/@clerk/clerk-js'));
    expect(hasClerkScript).toBe(true);
  });
});
