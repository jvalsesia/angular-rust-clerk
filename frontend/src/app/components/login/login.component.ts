import { Component, ElementRef, ViewChild, AfterViewInit, effect, inject } from '@angular/core';
import { ClerkService } from '../../services/clerk.service';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [MatProgressSpinnerModule],
  templateUrl: './login.component.html',
  styleUrl: './login.component.css'
})
export class LoginComponent implements AfterViewInit {
  private clerkService = inject(ClerkService);
  
  @ViewChild('clerkSignIn') clerkSignInContainer!: ElementRef<HTMLDivElement>;
  
  protected isClerkLoaded = this.clerkService.isLoaded;
  
  constructor() {
    // Dynamically mount when Clerk becomes ready
    effect(() => {
      if (this.isClerkLoaded() && this.clerkSignInContainer) {
        this.clerkService.mountSignIn(this.clerkSignInContainer.nativeElement);
      }
    });
  }

  ngAfterViewInit() {
    if (this.isClerkLoaded()) {
      this.clerkService.mountSignIn(this.clerkSignInContainer.nativeElement);
    }
  }
}
