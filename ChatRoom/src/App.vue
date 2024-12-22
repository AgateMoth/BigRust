<!-- filepath: /src/App.vue -->
<template>
  <div class="app-container">
    <aside class="sidebar">
      <h2>导航菜单</h2>
      <ul>
        <li>
          <router-link to="/">主页面</router-link>
        </li>
        <li>
          <router-link to="/chat">聊天室</router-link>
        </li>
      </ul>
    </aside>
    
    <main class="main-content">
      <router-view></router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
// 无需额外脚本
</script>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 200px;
  background-color: #2c3e50;
  color: white;
  padding: 20px;
}

.sidebar h2 {
  margin-top: 0;
}

.sidebar ul {
  list-style: none;
  padding: 0;
}

.sidebar li {
  margin: 15px 0;
}

.sidebar a {
  color: white;
  text-decoration: none;
}

.sidebar a.router-link-active {
  font-weight: bold;
}

.main-content {
  flex: 1;
  padding: 20px;
  background-color: #ecf0f1;
  overflow-y: auto;
}
</style>