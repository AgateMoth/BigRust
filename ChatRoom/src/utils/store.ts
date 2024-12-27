import { reactive } from 'vue';

export const store = reactive({
  isLoggedIn: false,
  username: '', 
});

export const mysql = reactive({
  mysqlname: 'root',
  mysqlpassword: '123',
  mysqlhost: '127.0.0.1',
  mysqlport: '3306',
  mysqldb: 'userdata',
});