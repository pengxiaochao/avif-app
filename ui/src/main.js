/**
 * main.js — 前端应用入口文件
 *
 * 这是整个 Vue 应用的启动入口。它做三件事：
 * 1. 从 Vue 框架中引入 createApp 函数（用于创建应用实例）
 * 2. 引入根组件 App.vue（整个界面的顶层容器）
 * 3. 引入全局 CSS 样式文件
 *
 * 最后调用 createApp(App).mount('#app') 将 Vue 应用"挂载"到
 * index.html 中 id="app" 的 <div> 元素上，界面就显示出来了。
 */
import { createApp } from 'vue'
import App from './App.vue'
import './style.css'

// 创建 Vue 应用实例并挂载到 HTML 页面中 id 为 "app" 的元素上
createApp(App).mount('#app')
