import { reactive } from 'vue';

export const store = reactive({
  isLoggedIn: false,
  username: '', // 新增用户名
});