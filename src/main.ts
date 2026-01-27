import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";

// Disable context menu for the app, but allow it for input fields
document.addEventListener('contextmenu', (event) => {
  const target = event.target as HTMLElement;
  const isInput = ['INPUT', 'TEXTAREA'].includes(target.tagName) || target.isContentEditable;
  
  // Also check if the input is not disabled or readonly if we want to be strict,
  // but generally allowing context menu on inputs is good UX even if readonly (for copy).
  if (!isInput) {
    event.preventDefault();
  }
});

const app = createApp(App);
app.use(createPinia());
app.mount("#app");
