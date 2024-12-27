<template>
  <div>
    <!-- 如果未登录，就只显示登录/注册页面 -->
    <RegisterView v-if="!store.isLoggedIn" />

    <!-- 如果已登录，才显示侧边栏和其他内容 -->
    <div v-else class="main-container">
      <aside class="sidebar">
        <div class="sidebar-header">
          <h2>导航</h2>
          <p class="username" style="color: #FFFDEC;">用户名： {{ store.username }}</p> <!-- 显示用户名 -->
        </div>
        <ul class="nav-links">
          <li>
            <router-link to="/home" class="nav-link">主页</router-link>
          </li>
          <li>
            <router-link to="/chat" class="nav-link">聊天</router-link>
          </li>
        </ul>
      </aside>
      <div class="content">
        <transition name="fade" mode="out-in">
          <router-view />
        </transition>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { store } from './utils/store';
import RegisterView from './views/RegisterView.vue';
</script>

<style scoped>
.main-container {
  display: flex;
  height: 100vh;
  background-color: #DFD0B8;
  animation: backgroundFade 1s ease-in-out;
}

/* 侧边栏样式 */
.sidebar {
  width: 250px;
  background-color: #153448;
  color: #DFD0B8;
  display: flex;
  flex-direction: column;
  padding: 20px;
  box-shadow: 2px 0 5px rgba(0, 0, 0, 0.1);
  animation: slideIn 0.5s forwards;
}

/* 侧边栏顶部 */
.sidebar-header {
  text-align: center;
  margin-bottom: 30px;
}

.sidebar-header h2 {
  color: #DFD0B8;
  font-size: 24px;
  transition: color 0.3s ease;
}

.sidebar-header h2:hover {
  color: #3C5B6F;
}

.username {
  margin-top: 10px;
  font-size: 16px;
  color: #DFD0B8;
  animation: fadeIn 1s ease;
}

/* 导航链接 */
.nav-links {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-links li {
  margin-bottom: 20px;
}

.nav-link {
  color: #DFD0B8;
  text-decoration: none;
  font-size: 18px;
  position: relative;
  transition: color 0.3s ease;
}

.nav-link::after {
  content: '';
  display: block;
  width: 0;
  height: 2px;
  background: #3C5B6F;
  transition: width 0.3s;
  position: absolute;
  bottom: -5px;
  left: 0;
}

.nav-link:hover {
  color: #3C5B6F;
}

.nav-link:hover::after {
  width: 100%;
}

/* 内容区域 */
.content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

/* 页面切换淡入淡出效果 */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.5s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

/* 动画关键帧 */
@keyframes slideIn {
  from {
    transform: translateX(-250px);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

@keyframes backgroundFade {
  from {
    background-color: #3C5B6F;
  }
  to {
    background-color: #DFD0B8;
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .main-container {
    flex-direction: column;
  }
  
  .sidebar {
    width: 100%;
    flex-direction: row;
    justify-content: space-around;
    align-items: center;
    animation: none;
    box-shadow: none;
  }

  .sidebar-header {
    display: none;
  }

  .nav-links {
    display: flex;
    gap: 20px;
  }

  .nav-links li {
    margin-bottom: 0;
  }
}
</style>