<template>
  <div class="container">
    <h1 class="title">聊天室</h1>
    
    <div class="chat-layout">
      <!-- 配置设置区域 -->
      <div class="config-dropdown">
        <button @click="toggleConfig" class="dropdown-button">配置设置</button>
        <transition name="fade">
          <div v-if="showConfig" class="config-menu">
            <div class="config-section">
              <div class="config-group">
                <label for="localPort">本地监听端口:</label>
                <input id="localPort" v-model="localPort" type="number" min="1" max="65535" />
                <button @click="setLocalPort">设置端口</button>
              </div>
              <p class="current-port">当前监听端口: {{ currentPort }}</p>
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
      
      <!-- 收到的消息区域 -->
      <div class="messages-section">
        <div v-if="receivedMessages.length" class="messages-list">
          <transition-group name="message" tag="div">
            <div
              v-for="(msg, index) in receivedMessages"
              :key="index"
              :class="['message-item', msg.username === store.username ? 'self' : 'others']"
            >
              <strong>{{ msg.username }}:</strong> {{ msg.content }}
            </div>
          </transition-group>
        </div>
        <div v-else class="no-messages">没有收到消息</div>
      </div>
      
      <!-- 发送消息区域 -->
      <div class="send-section">
        <textarea v-model="message" placeholder="输入消息..."></textarea>
        <button @click="send">发送</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { store } from '../utils/store';
import { ElMessage } from 'element-plus';
import 'element-plus/dist/index.css';

interface ReceivedMessage {
  username: string;
  content: string;
}

const showConfig = ref(false);
const target = ref('');
const port = ref(8080);
const message = ref('');
const receivedMessages = ref<ReceivedMessage[]>([]);
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
      ElMessage({
        message: '设置成功，请重新启动应用',
        type: 'success',
      });
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
        username: store.username, // 发送用户名
        message: message.value, 
        target: target.value, 
        port: port.value 
      });
      message.value = '';
    } catch (error) {
      console.error('发送消息失败:', error);
    }
  } else {
    ElMessage({
      message: '请在配置中填写所有信息或为写任何消息',
      type: 'warning',
    });
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
    try {
      const msg: ReceivedMessage = JSON.parse(event.payload);
      receivedMessages.value.push(msg);
    } catch (e) {
      console.error('解析消息失败:', e);
    }
  }).catch((error) => {
    console.error('监听消息失败:', error);
  });
});
</script>

<style scoped>
.container {
  height: 96vh;
  overflow: hidden; 
  background-color: #F5EFE7;
  display: flex;
  flex-direction: column;
  align-items: center;
  font-family: Arial, sans-serif;
  color: #213555;
}


.title {
  color: #3E5879;
  margin-top: 20px;
}

.chat-layout {
  flex: 1;
  width: 90%;
  max-width: 800px;
  margin: 20px;
  display: flex;
  flex-direction: column;
  background-color: #D8C4B6;
  border-radius: 10px;
  padding: 20px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  overflow: hidden; 
}

.config-dropdown {
  position: relative;
  margin-bottom: 20px;
}

.dropdown-button {
  background-color: #3E5879;
  color: #F5EFE7;
  border: none;
  padding: 10px 20px;
  border-radius: 5px;
  cursor: pointer;
}

.config-menu {
  position: absolute;
  top: 40px;
  left: 0;
  background-color: #FFFFFF;
  border: 1px solid #213555;
  border-radius: 5px;
  padding: 10px;
  z-index: 100;
}

.config-group {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.config-group label {
  margin-right: 10px;
  color: #213555;
}

.config-group input {
  flex: 1;
  padding: 5px;
  border: 1px solid #3E5879;
  border-radius: 3px;
}

.config-group button {
  margin-left: 10px;
  background-color: #3E5879;
  color: #F5EFE7;
  border: none;
  padding: 5px 10px;
  border-radius: 3px;
  cursor: pointer;
}

.current-port {
  color: #213555;
}

.messages-section {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #3E5879;
  border-radius: 5px;
  padding: 10px;
  background-color: #FFFFFF;
  margin-bottom: 20px;
}

.messages-list {
  display: flex;
  flex-direction: column;
}

.message-item {
  max-width: 70%;
  padding: 10px;
  margin: 5px 0;
  border-radius: 10px;
  word-wrap: break-word;
  color: #213555;
}

.message-item.self {
  align-self: flex-end;
  background-color: #3E5879;
  color: #F5EFE7;
}

.message-item.others {
  align-self: flex-start;
  background-color: #D8C4B6;
}

.no-messages {
  text-align: center;
  color: #213555;
}

.send-section {
  display: flex;
  align-items: center;
}

.send-section textarea {
  flex: 1;
  padding: 10px;
  border: 1px solid #3E5879;
  border-radius: 5px;
  resize: none;
  height: 60px;
  color: #213555;
}

.send-section button {
  margin-left: 10px;
  background-color: #3E5879;
  color: #F5EFE7;
  border: none;
  padding: 10px 20px;
  border-radius: 5px;
  cursor: pointer;
}
</style>