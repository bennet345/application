import { Component, EventEmitter, Input, Output } from '@angular/core';
import { FormsModule } from '@angular/forms';

interface Entry {
  done: boolean,
  content: string,
}

@Component({
  selector: 'todo-entry',
  imports: [FormsModule],
  template: `
    <div>
      {{ id + 1 }}.
      <input type='checkbox' [checked]='entry.done' (change)='toggle.emit()'/>
      @if (!editing) {
        <span (click)='showMenu = !showMenu'>{{ entry.content }}</span>
      } @else {
        <input [(ngModel)]='newContent'/>
      }
      @if (showMenu) {
        @if (!editing) {
          <button (click)='editing = !editing'>edit</button>
        } @else {
          <button (click)='emitEdit()'>update</button>
          <button (click)='editing = !editing'>cancel</button>
        }
        <button (click)='delete.emit()'>delete</button>
      }
    </div>
  `,
  styles: ``,
})
export class TodoEntry {
  @Input() entry!: Entry;
  @Input() id!: number;
  @Output() toggle = new EventEmitter<void>();
  @Output() delete = new EventEmitter<void>();
  @Output() edit = new EventEmitter<string>();
  showMenu: boolean = false;
  editing: boolean = false;
  newContent: string = '';

  ngOnInit(): void {
    this.newContent = this.entry.content;
  }

  emitEdit() {
    this.edit.emit(this.newContent);
    this.editing = false;
    this.showMenu = false;
  }
}

@Component({
  selector: 'app-todo',
  imports: [TodoEntry, FormsModule],
  template: `
    <input [(ngModel)]='newEntry'/>
    @if (newEntry.length > 0) {
      <button (click)='addEntry(newEntry)'>add</button>
    }
    @if (countDoneEntries() > 0) {
      <button (click)='clean()'>clean</button>
    }
    @if (entries.length > 0) {
      <button (click)='clear()'>clear</button>
    }

    @for (entry of entries; track $index) {
      <todo-entry
        [entry]='entry'
        [id]='$index'
        (toggle)='toggleEntryDone($index)'
        (delete)='deleteEntry($index)'
        (edit)='editEntry($index, $event)'
      />
    }
  `,
  styles: ``,
})
export class TodoComponent {
  entries: Entry[] = [];
  newEntry: string = '';

  addEntry(content: string) {
    this.entries.push({
      done: false,
      content,
    });
    this.newEntry = '';
  }

  toggleEntryDone(id: number) {
    this.entries[id].done = !this.entries[id].done;
  }

  editEntry(id: number, content: string) {
    this.entries[id].content = content;
  }

  deleteEntry(id: number) {
    this.entries.splice(id, 1);
  }

  clean() {
    this.entries = this.entries.filter(entry => ! entry.done);
  }

  clear() {
    this.entries = [];
  }

  countDoneEntries(): number {
    return this.entries.filter(entry => entry.done).length;
  }
}
