<template>
    <div class="container">
      <h1 class="title">聊天室</h1>
      <div class="settings">
        <input v-model="target" placeholder="目标地址" class="input" />
        <input v-model.number="port" type="number" placeholder="目标端口" class="input" />
      </div>
      <div class="message-section">
        <textarea v-model="message" placeholder="输入消息" class="textarea"></textarea>
        <button @click="send" class="button">发送</button>
      </div>
      <div class="received-messages">
        <h2>接收消息</h2>
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
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.1);
  }
  
  .title {
    text-align: center;
    color: #333;
    margin-bottom: 20px;
  }
  
  .settings,
  .message-section {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
  }
  
  .input {
    flex: 1;
    padding: 10px;
    border: 1px solid #ccc;
    border-radius: 4px;
  }
  
  .textarea {
    flex: 1;
    padding: 10px;
    height: 100px;
    border: 1px solid #ccc;
    border-radius: 4px;
    resize: none;
  }
  
  .button {
    padding: 10px 20px;
    background-color: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.3s;
  }
  
  .button:hover {
    background-color: #0056b3;
  }
  
  .received-messages {
    border-top: 1px solid #ccc;
    padding-top: 20px;
  }
  
  .message {
    background-color: #e1f5fe;
    padding: 10px;
    margin-bottom: 10px;
    border-radius: 4px;
    word-wrap: break-word;
  }
  </style>