import { Component, HostListener, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ServerService } from '../server.service';

interface Food {
  position: number[],
  time: number,
}

@Component({
  selector: 'app-snake',
  imports: [FormsModule],
  template: `
    <div class='board'>
      @for (field of grid(); track $index) {
        <span class='field {{ field }}'></span>
        @if (($index + 1) % 10 === 0) { <div></div> }
    }
    </div>
    <div>
      Difficulty
      <input
        type='range'
        min='1' max='100'
        [(ngModel)]='difficulty'
      />
    </div>
    @if (server.score !== undefined) {
      <div>Score: {{ server.score }}</div>
    }
  `,
  styles: `
    .board {
      font-size: 0;
    }

    .field {
      display: inline-block;
      width: 30px;
      height: 30px;
    }

    .snake  { background-color: red; }
    .food   { background-color: green; }
    .empty  { background-color: rgb(64, 128, 255); }
  `
})
export class SnakeComponent {
  server: ServerService = inject(ServerService);
  snake: number[][] = [[0, 0], [0, 1]];
  direction: number[] = [0, 1];
  food: Food[] = [];
  difficulty: number = 50;

  async ngOnInit() {
    let iteration = 0;
    while (true) {
      this.forward();
      if (iteration % 5 === 0) {
        this.food.push({
          position: this.unoccupiedPosition(),
          time: iteration,
        });
      }
      for (let i = 0; i < this.food.length; i++) {
        if (iteration - this.food[this.food.length - i - 1].time >= 25) {
          this.food.splice(this.food.length - i - 1, 1);
        }
      }
      iteration += 1;
      await new Promise(resolve => setTimeout(resolve, 400 - this.difficulty * 3));
    }
  }

  @HostListener('window:keyup', ['$event'])
  keyEvent(event: KeyboardEvent) {
    switch (event.key) {
      case 'w': { this.setDirection([0, -1]); break; }
      case 's': { this.setDirection([0, 1]);  break; }
      case 'd': { this.setDirection([1, 0]);  break; }
      case 'a': { this.setDirection([-1, 0]); break; }
    }
  }

  forward() {
    let newPosition = [
      this.modulo((this.snake[this.snake.length - 1][0] + this.direction[0]), 10),
      this.modulo((this.snake[this.snake.length - 1][1] + this.direction[1]), 10),
    ];
    let element = this.snake.shift()!;
    switch (this.gridPosition(newPosition[0], newPosition[1])) {
      case 'food': {
        for (let i = 0; i < this.food.length; i++) {
          if (this.food[i].position[0] === newPosition[0] && this.food[i].position[1] === newPosition[1]) {
            this.food.splice(i, 1);
            break;
          }
        }
        this.server.incrementScore();
        this.snake.unshift(element);
        if (element[0] === newPosition[0] && element[1] === newPosition[1]) {
          this.die();
        }
        break;
      }
      case 'snake': {
        this.die();
        break;

      }
    }
    this.snake.push(newPosition);
  }

  die() {
    this.snake = [this.snake[this.snake.length - 1]];
  }

  unoccupiedPosition(): number[] {
    while (true) {
      let position = [Math.floor(Math.random() * 10), Math.floor(Math.random() * 10)];
      if (this.gridPosition(position[0], position[1]) === 'empty') return position;
    }
  }

  grid(): string[] {
    let output: string[] = [];
    for (let y = 0; y < 10; y++) {
      for (let x = 0; x < 10; x++) {
        output.push(this.gridPosition(x, y));
      }
    }
    return output;
  }

  gridPosition(x: number, y: number): string {
      for (let part of this.snake) {
        if (part[0] === x && part[1] === y) {
          return 'snake';
        }
      }
      for (let food of this.food) {
        if (food.position[0] === x && food.position[1] === y) {
          return 'food';
        }
      }
      return 'empty';
  }

  setDirection(direction: number[]) {
    let illegalDirection = [
      -(this.snake[this.snake.length - 1][0] - this.snake[this.snake.length - 2][0]),
      -(this.snake[this.snake.length - 1][1] - this.snake[this.snake.length - 2][1]),
    ];
    if (direction[0] === illegalDirection[0] && direction[1] === illegalDirection[1]) {
      return;
    }
    this.direction = direction;
  }

  modulo(value: number, x: number): number {
    return ((value % x) + x) % x;
  }
}
