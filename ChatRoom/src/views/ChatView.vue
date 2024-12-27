<template>
  <div class="container">
    <h1 class="title">聊天室</h1>
    
    <div class="config-dropdown">
      <button @click="toggleConfig" class="dropdown-button">配置设置</button>
      <transition name="fade">
        <div v-if="showConfig" class="config-menu">
          <div class="config-section">
            <div class="config-group">
              <label for="localPort">本地端口:</label>
              <input id="localPort" v-model="localPort" type="number" min="1" max="65535" />
              <button @click="setLocalPort">设置端口</button>
            </div>
            <p class="current-port">当前端口: {{ currentPort }}</p>
          </div>
          
          <div class="config-section">
            <div class="config-group">
              <label for="target">目标地址:</label>
              <input id="target" v-model="target" type="text" placeholder="例如: 127.0.0.1" />
            </div>
            <div class="config-group">
              <label for="port">目标端口:</label>
              <input id="port" v-model="port" type="number" min="1" max="65535" />
            </div>
          </div>
        </div>
      </transition>
    </div>
    
    <div class="send-section">
      <textarea v-model="message" placeholder="输入消息..."></textarea>
      <button @click="send">发送</button>
    </div>
    
    <div class="messages-section">
      <h2>收到的消息</h2>
      <div v-if="receivedMessages.length" class="messages-list">
        <div v-for="(msg, index) in receivedMessages" :key="index" class="message-item">
          {{ msg }}
        </div>
      </div>
      <div v-else class="no-messages">没有收到消息</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const showConfig = ref(false);
const target = ref('');
const port = ref(8080);
const message = ref('');
const receivedMessages = ref<string[]>([]);
const localPort = ref(8080);
const currentPort = ref(8080);

// 切换配置菜单显示
const toggleConfig = () => {
  showConfig.value = !showConfig.value;
};

// 设置本地端口
const setLocalPort = async () => {
  if (localPort.value >= 1 && localPort.value <= 65535) {
    try {
      await invoke('set_local_port', { port: localPort.value });
      console.log(`本地端口已设置为: ${localPort.value}`);
      currentPort.value = localPort.value;
      alert('设置成功，请重新启动应用');
    } catch (error) {
      console.error('设置本地端口失败:', error);
    }
  } else {
    console.error('端口号无效');
  }
};

// 发送消息
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

// 获取当前端口
const fetchCurrentPort = async () => {
  try {
    const port = await invoke('get_local_port') as number;
    currentPort.value = port;
    localPort.value = port;
  } catch (error) {
    console.error('获取当前端口失败:', error);
  }
};

onMounted(() => {
  fetchCurrentPort();

  listen<string>('receive_message', (event) => {
    receivedMessages.value.push(event.payload);
  }).catch((error) => {
    console.error('监听消息失败:', error);
  });
});
</script>

<style scoped>
:root {
  --color1: #153448;
  --color2: #3C5B6F;
  --color3: #948979;
  --color4: #DFD0B8;
}

.container {
  max-width: 900px;
  margin: 20px auto;
  padding: 25px;
  background-color: var(--color4);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  border-radius: 10px;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  animation: fadeIn 1s ease-in;
}

.title {
  text-align: center;
  color: var(--color1);
  margin-bottom: 30px;
  animation: slideDown 1s ease-out;
}

.config-dropdown {
  margin-bottom: 25px;
}

.dropdown-button {
  background-color: var(--color2);
  color: white;
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.dropdown-button:hover {
  background-color: var(--color3);
}

.config-menu {
  background-color: var(--color4);
  padding: 15px;
  border: 1px solid var(--color2);
  border-radius: 8px;
  animation: fadeIn 0.5s ease-in;
}

.config-section {
  margin-bottom: 15px;
}

.config-group {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.config-group label {
  width: 100px;
  font-weight: bold;
  color: var(--color1);
}

.config-group input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #bdc3c7;
  border-radius: 4px;
  margin-right: 10px;
  transition: border-color 0.3s;
}

.config-group input:focus {
  border-color: var(--color2);
  outline: none;
}

.config-group button {
  padding: 8px 16px;
  background-color: var(--color2);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.config-group button:hover {
  background-color: var(--color3);
}

.current-port {
  text-align: right;
  font-style: italic;
  color: var(--color3);
}

.send-section {
  display: flex;
  flex-direction: column;
  margin-bottom: 25px;
}

.send-section textarea {
  resize: none;
  height: 120px;
  padding: 12px;
  border: 1px solid #bdc3c7;
  border-radius: 4px;
  margin-bottom: 10px;
  transition: border-color 0.3s;
}

.send-section textarea:focus {
  border-color: var(--color1);
  outline: none;
}

.send-section button {
  align-self: flex-end;
  padding: 10px 20px;
  background-color: var(--color1);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.send-section button:hover {
  background-color: #10202b;
}

.messages-section {
  padding: 15px;
  border: 1px solid #ecf0f1;
  border-radius: 8px;
  background-color: #f7f9fc;
}

.messages-section h2 {
  margin-bottom: 15px;
  color: var(--color1);
  text-align: center;
}

.messages-list {
  max-height: 300px;
  overflow-y: auto;
}

.message-item {
  background-color: #dff9fb;
  padding: 10px 15px;
  border-radius: 6px;
  margin-bottom: 10px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  transition: background-color 0.3s;
}

.message-item:hover {
  background-color: #cdeefb;
}

.no-messages {
  text-align: center;
  color: #95a5a6;
  font-style: italic;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideDown {
  from { transform: translateY(-20px); }
  to { transform: translateY(0); }
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.5s;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>