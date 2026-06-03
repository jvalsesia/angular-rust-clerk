import { Component, inject, OnInit, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterLink } from '@angular/router';
import { AuthService } from '../../services/auth.service';
import { DataService, ProtectedData } from '../../services/data.service';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './dashboard.component.html',
  styleUrls: ['./dashboard.component.css']
})
export class DashboardComponent implements OnInit {
  private authService = inject(AuthService);
  private dataService = inject(DataService);
  private router = inject(Router);

  public user = this.authService.user;
  public protectedData = signal<ProtectedData | null>(null);
  public loadError = signal<string | null>(null);

  public ngOnInit() {
    this.dataService.getProtectedData().subscribe({
      next: (data) => this.protectedData.set(data),
      error: () => this.loadError.set('Failed to fetch secure workspace information.')
    });
  }

  public async onSignOut() {
    await this.authService.signOut();
    this.router.navigate(['/']);
  }
}
