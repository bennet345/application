import { Component, inject } from '@angular/core';
import { RouterModule, RouterOutlet } from '@angular/router';
import { ServerService } from './server.service';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, RouterModule],
  template: `
    <div>
      <span [routerLink]="['/todo']" style='margin-right: 5px;'>ToDo</span>
      <span [routerLink]="['/snake']">Snake</span>
    </div>
    <router-outlet />
  `,
})
export class AppComponent {
  title = 'site';
  server: ServerService = inject(ServerService);
}
