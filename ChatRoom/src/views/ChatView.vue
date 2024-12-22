<!-- filepath: /src/views/ChatView.vue -->
<template>
    <div class="container">
      <h1>聊天室</h1>
      
      <div class="config">
        <label for="localPort">本地端口:</label>
        <input id="localPort" v-model="localPort" type="number" min="1" max="65535" />
        <button @click="setLocalPort">设置端口</button>
      </div>
      
      <div class="config">
        <label for="target">目标地址:</label>
        <input id="target" v-model="target" type="text" placeholder="例如: 127.0.0.1" />
        
        <label for="port">目标端口:</label>
        <input id="port" v-model="port" type="number" min="1" max="65535" />
      </div>
      
      <div class="send-message">
        <textarea v-model="message" placeholder="输入消息..."></textarea>
        <button @click="send">发送</button>
      </div>
      
      <div class="messages">
        <div v-for="(msg, index) in receivedMessages" :key="index" class="message">{{ msg }}</div>
      </div>
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref, onMounted } from 'vue';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  
  const target = ref('');
  const port = ref(8080);
  const message = ref('');
  const receivedMessages = ref<string[]>([]);
  const localPort = ref(8080);
  
  // 设置本地端口
  const setLocalPort = async () => {
    if (localPort.value >= 1 && localPort.value <= 65535) {
      try {
        await invoke('set_local_port', {port:localPort.value});
        console.log(`本地端口已设置为: ${localPort.value}`);
        // 重新加载应用以应用新的端口设置
        window.location.reload();
      } catch (error) {
        console.error('设置本地端口失败:', error);
      }
    } else {
      console.error('端口号无效');
    }
  };
  
  const send = async () => {
    if (target.value && port.value && message.value) {
      try {
        await invoke('send_message', { 
          message: message.value, 
          target: target.value, 
          port: port.value 
        });
        message.value = '';
      } catch (error) {
        console.error('发送消息失败:', error);
      }
    }
  };
  
  onMounted(() => {
    listen<string>('receive_message', (event) => {
      receivedMessages.value.push(event.payload);
    }).catch((error) => {
      console.error('监听消息失败:', error);
    });
  });
  </script>
  
  <style scoped>
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background-color: #f9f9f9;
    border-radius: 8px;
  }
  
  .config {
    margin-bottom: 20px;
  }
  
  .config label {
    margin-right: 10px;
  }
  
  .send-message {
    display: flex;
    flex-direction: column;
  }
  
  .send-message textarea {
    resize: none;
    height: 100px;
    margin-bottom: 10px;
  }
  
  .send-message button {
    align-self: flex-end;
  }
  
  .messages {
    margin-top: 20px;
  }
  
  .message {
    background-color: #e1f5fe;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 10px;
  }
  </style>