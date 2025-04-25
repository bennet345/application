import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root'
})
export class ServerService {
  socket: WebSocket = new WebSocket('ws://localhost:8080/echo');
  score: number | undefined;

  constructor() {
    let listener = async (event: MessageEvent<any>) => {
      this.score = Number(event.data);
      //this.socket.removeEventListener('message', listener);
    };
    this.socket.addEventListener('message', listener);
  }

  incrementScore() {
    this.socket.send("");
  }
}
