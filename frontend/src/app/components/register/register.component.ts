import { Component, ElementRef, ViewChild, AfterViewInit, effect, inject } from '@angular/core';
import { ClerkService } from '../../services/clerk.service';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

@Component({
  selector: 'app-register',
  standalone: true,
  imports: [MatProgressSpinnerModule],
  templateUrl: './register.component.html',
  styleUrl: './register.component.css'
})
export class RegisterComponent implements AfterViewInit {
  private clerkService = inject(ClerkService);
  
  @ViewChild('clerkSignUp') clerkSignUpContainer!: ElementRef<HTMLDivElement>;
  
  protected isClerkLoaded = this.clerkService.isLoaded;
  
  constructor() {
    effect(() => {
      if (this.isClerkLoaded() && this.clerkSignUpContainer) {
        this.clerkService.mountSignUp(this.clerkSignUpContainer.nativeElement);
      }
    });
  }

  ngAfterViewInit() {
    if (this.isClerkLoaded()) {
      this.clerkService.mountSignUp(this.clerkSignUpContainer.nativeElement);
    }
  }
}
