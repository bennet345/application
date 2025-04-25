import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SnakeComponent } from './snake.component';

describe('SnakeComponent', () => {
  let component: SnakeComponent;
  let fixture: ComponentFixture<SnakeComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [SnakeComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(SnakeComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should get the non-negative modulo', () => {
    expect(component.modulo(-1, 2)).toBe(1);
  });

  it('should identify what\'s on a field', () => {
    component.food = [{
      position: [0, 0],
      time: 0,
    }];
    component.snake = [[1, 1], [1, 2]];
    expect(component.gridPosition(0, 0)).toBe('food');
    expect(component.gridPosition(1, 2)).toBe('snake');
    expect(component.gridPosition(4, 4)).toBe('empty');
  });

  it('shouldn\'t allow going in the opposite direction', () => {
    component.food = [{
      position: [0, 0],
      time: 0,
    }];
    component.snake = [[1, 1], [1, 2]];
    component.direction = [0, 1];
    component.setDirection([0, -1]);
    expect(component.direction[1]).toBe(1);
    component.setDirection([1, 0]);
    expect(component.direction[1]).toBe(0);
  })
});
