import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TodoComponent } from './todo.component';

describe('TodoComponent', () => {
  let component: TodoComponent;
  let fixture: ComponentFixture<TodoComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TodoComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(TodoComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should clean entries that are done', () => {
    component.addEntry('one');
    component.addEntry('two');
    component.addEntry('three');
    component.toggleEntryDone(0);
    component.toggleEntryDone(2);
    component.clean();
    expect(component.entries.length).toBe(1);
    expect(component.entries[0].content).toBe('two');
  });
});
