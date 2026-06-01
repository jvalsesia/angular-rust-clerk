import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

export interface ProtectedData {
  message: string;
  user_id: string;
  timestamp: number;
}

@Injectable({
  providedIn: 'root'
})
export class DataService {
  private http = inject(HttpClient);

  public getProtectedData(): Observable<ProtectedData> {
    return this.http.get<ProtectedData>('/api/protected');
  }
}
