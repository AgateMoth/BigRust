<template>
  <div class="home-container">
    <!-- 欢迎信息 -->
    <div class="welcome-section" @mouseover="animateSection('welcome')" @mouseleave="resetSection('welcome')">
      <h1 class="welcome-title">欢迎回来，{{ store.username }}！</h1>
      <p class="welcome-message">准备开始与大家聊天吧！</p>
    </div>

    <!-- 功能按钮 -->
    <div class="actions-section" @mouseover="animateSection('actions')" @mouseleave="resetSection('actions')">
      <button @click="navigateToChat" class="action-button">进入聊天室</button>
      <button @click="logout" class="action-button">退出登录</button>
    </div>

    <!-- 用户信息 -->
    <div class="user-info" @mouseover="animateSection('info')" @mouseleave="resetSection('info')">
      <p><strong>用户名：</strong> {{ store.username }}</p>
      <p><strong>在线状态：</strong> 在线</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { ref } from 'vue';
import { store } from '../utils/store';

// 导航到聊天室
const router = useRouter();

// 定义一个类型以确保 animatedSections 的结构
type SectionKeys = 'welcome' | 'actions' | 'info';

const animatedSections = ref<Record<SectionKeys, boolean>>({
  welcome: false,
  actions: false,
  info: false
});

function animateSection(section: SectionKeys) {
  animatedSections.value[section] = true;
}

function resetSection(section: SectionKeys) {
  animatedSections.value[section] = false;
}

// 导航到聊天室
const navigateToChat = () => {
  router.push('/chat');
};

// 用户登出
const logout = () => {
  store.username = ''; // 清空用户名
  router.push('/'); // 跳转到登录/注册页面
};
</script>

<style scoped>
.home-container {
  background-color: #F5EFE7;
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  font-family: Arial, sans-serif;
  color: #213555;
  justify-content: center;
  padding: 20px;
}

.welcome-section {
  text-align: center;
  margin-bottom: 40px;
  transition: transform 0.3s ease, background-color 0.5s ease;
  padding: 20px;
  border-radius: 12px;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
}

.welcome-section:hover {
  transform: scale(1.05);
  background-color: #D8C4B6;
}

.welcome-title {
  color: #3E5879;
  font-size: 2rem;
  animation: fadeIn 0.5s ease;
}

.welcome-message {
  color: #213555;
  font-size: 1.2rem;
  animation: fadeIn 0.5s ease;
}

.actions-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 40px;
  transition: transform 0.3s ease, background-color 0.5s ease;
  padding: 20px;
  border-radius: 12px;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
}

.actions-section:hover {
  transform: scale(1.05);
  background-color: #D8C4B6;
}

.action-button {
  background-color: #3E5879;
  color: #F5EFE7;
  padding: 10px 20px;
  margin: 10px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  font-size: 1rem;
  width: 200px;
  transition: background-color 0.3s ease, transform 0.2s ease;
}

.action-button:hover {
  background-color: #F5EFE7;
  color: #3C5B6F;
  border: 1px solid #3E5879;
  transform: translateY(-2px);
}

.user-info {
  text-align: center;
  background-color: #D8C4B6;
  padding: 20px;
  border-radius: 10px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  transition: transform 0.3s ease, background-color 0.5s ease;
}

.user-info:hover {
  transform: scale(1.05);
  background-color: #F5EFE7;
}

.user-info p {
  margin: 10px 0;
  animation: fadeIn 0.5s ease;
}

.user-info strong {
  color: #3E5879;
}

/* 动画关键帧 */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>