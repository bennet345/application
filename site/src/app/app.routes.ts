import { Routes } from '@angular/router';
import { TodoComponent } from './todo/todo.component';
import { SnakeComponent } from './snake/snake.component';

export const routes: Routes = [
  {
    path: 'todo',
    component: TodoComponent,
    title: 'ToDo',
  },
  {
    path: 'snake',
    component: SnakeComponent,
    title: 'Snake',
  },
];
