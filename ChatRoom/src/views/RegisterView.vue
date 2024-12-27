<!-- filepath: /src/views/RegisterView.vue -->
<template>
  <div class="container">
    <div class="form-container" @mouseover="animateForm" @mouseleave="resetForm">
      <h2 v-if="!isRegister" class="title">登录</h2>
      <h2 v-else class="title">注册</h2>
      <form @submit.prevent="handleSubmit">
        <div class="form-group">
          <label>用户名：</label>
          <input v-model="username" type="text" required />
        </div>
        <div class="form-group">
          <label>密码：</label>
          <input v-model="password" type="password" required />
        </div>
        <div v-if="isRegister" class="form-group">
          <label>确认密码：</label>
          <input v-model="confirmPassword" type="password" required />
        </div>
        <button type="submit" class="submit-btn">{{ isRegister ? '注册' : '登录' }}</button>
      </form>
      <button @click="toggleRegister" class="toggle-btn">
        {{ isRegister ? '已有账号？去登录' : '没有账号？去注册' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { store } from '../utils/store';

const router = useRouter();

const username = ref('');
const password = ref('');
const confirmPassword = ref('');
const isRegister = ref(false);

// 动画状态
const animate = ref(false);

function toggleRegister() {
  isRegister.value = !isRegister.value;
  password.value = '';
  confirmPassword.value = '';
}

async function handleSubmit() {
  if (isRegister.value) {
    if (password.value !== confirmPassword.value) {
      alert('两次输入的密码不一致');
      return;
    }
    try {
      await invoke('createuser_in_mysql', { 
        mysqlname: 'root',
        mysqlpassword: '123',
        mysqlhost: '127.0.0.1',
        mysqlport: '3306',
        mysqldb: 'userdata',
        username: username.value, 
        password: password.value 
      });
      alert('注册成功');
    } catch (error) {
      console.error('注册失败', error);
    }
  } else {
    try {
      const result = await invoke('find_in_mysql', { 
        mysqlname: 'root',
        mysqlpassword: '123',
        mysqlhost: '127.0.0.1',
        mysqlport: '3306',
        mysqldb: 'userdata',
        username: username.value,
        password: password.value 
      }) as boolean;
      if (result) {
        store.isLoggedIn = true; // 更新全局登录状态
        store.username = username.value; // 设置用户名
        router.push('/home');
      } else {
        alert('登录失败，请检查用户名或密码');
      }
    } catch (error) {
      console.error('登录失败', error);
    }
  }
}

// 动画控制函数
function animateForm() {
  animate.value = true;
}

function resetForm() {
  animate.value = false;
}
</script>

<style scoped>
.container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  background-color: #DFD0B8;
  transition: background-color 0.5s ease;
}

.form-container {
  background-color: #153448;
  padding: 30px;
  border-radius: 12px;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
  width: 350px;
  transform: scale(1);
  transition: transform 0.3s ease, background-color 0.5s ease;
}

.form-container:hover {
  transform: scale(1.05);
  background-color: #3C5B6F;
}

.title {
  text-align: center;
  color: #DFD0B8;
  margin-bottom: 20px;
  transition: color 0.3s ease;
}

.form-group {
  margin-bottom: 15px;
  animation: fadeIn 0.5s ease;
}

label {
  display: block;
  color: #948979;
  margin-bottom: 5px;
  font-weight: bold;
  transition: color 0.3s ease;
}

input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #948979;
  border-radius: 6px;
  box-sizing: border-box;
  transition: border-color 0.3s ease, box-shadow 0.3s ease;
}

input:focus {
  border-color: #DFD0B8;
  box-shadow: 0 0 5px rgba(223, 208, 184, 0.5);
  outline: none;
}

.submit-btn {
  width: 100%;
  padding: 12px;
  background-color: #3C5B6F;
  border: none;
  border-radius: 6px;
  color: #DFD0B8;
  font-weight: bold;
  cursor: pointer;
  transition: background-color 0.3s ease, transform 0.2s ease;
}

.submit-btn:hover {
  background-color: #153448;
  transform: translateY(-2px);
}

.toggle-btn {
  width: 100%;
  padding: 10px;
  background-color: transparent;
  border: none;
  color: #DFD0B8;
  cursor: pointer;
  text-decoration: underline;
  margin-top: 15px;
  transition: color 0.3s ease;
}

.toggle-btn:hover {
  color: #948979;
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