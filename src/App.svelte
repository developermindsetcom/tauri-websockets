<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke} from "@tauri-apps/api/core"

  let ws:WebSocket;
  let text = ''

  onMount(async () => {
    invoke("listen")
    ws = new WebSocket('ws://127.0.0.1:9001');

    ws.addEventListener('open', onOpen);
    ws.addEventListener('message', onMessage);
    ws.addEventListener('close', onClose);
    ws.addEventListener('error', onError);
  });

  onDestroy(async () => {
    if(ws){
      ws.removeEventListener('open', onOpen);
      ws.removeEventListener('message', onMessage);
      ws.removeEventListener('close', onClose);
      ws.removeEventListener('error', onError);
    }
  });

  const onOpen = (event:Event) => {
    console.log('open', event);
    text += `${getTimestamp()} Opened\n`;
  }

  const onMessage = (event:MessageEvent<String>) => {
    const received = event.data;
    text += `${getTimestamp()} Received: ${received}\n`;
  }

  const onClose = (event:CloseEvent) => {
    console.log('close', event);
    text += `${getTimestamp()} Closed ${event.reason}\n`;
  }

  const onError = (event:Event) => {
    console.error(event);
    text += `${getTimestamp()} Error ${event}}\n`;
  }

  let message = '';
  const send = () => {
    if(ws){
      ws.send(message);
      text += `${getTimestamp()} Sent: ${message}\n`;
      message = '';
    }
  }

  const getTimestamp = () => {
    const d = new Date();
    const hh = d.getHours();
    const mm = d.getMinutes();
    const ss = d.getSeconds();

    const pad = (n:number) => n < 10 ? '0' + n : n;

    return [
      pad(hh),
      pad(mm),
      pad(ss)
    ].join(':')
  }
  
</script>

<main class="container">
  <h1>WebSocket Client Demo by DeveloperMindset.com</h1>

  <form class="row" on:submit|preventDefault={send}>
    <input id="greet-input" placeholder="Enter a message..." bind:value={message} />
    <button type="submit">Send</button>
  </form>

  <pre>
    {text}
  </pre>

</main>

<style>
  pre {
    display: flex;
    flex-direction: column;
    justify-content: start;
    text-align: left;
  }
</style>