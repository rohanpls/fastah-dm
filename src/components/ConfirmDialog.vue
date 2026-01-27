<script setup lang="ts">
defineProps<{
  show: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}>();

const emit = defineEmits(['confirm', 'cancel']);
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="modal-overlay" @click.self="emit('cancel')">
        <div class="confirm-dialog glass-panel">
          <div class="dialog-icon" :class="{ danger }">
            <svg v-if="danger" xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
              <line x1="12" y1="9" x2="12" y2="13"></line>
              <line x1="12" y1="17" x2="12.01" y2="17"></line>
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
          </div>
          
          <h3>{{ title }}</h3>
          <p>{{ message }}</p>
          
          <slot name="extra"></slot>
          
          <div class="dialog-actions">
            <button class="btn btn-secondary" @click="emit('cancel')">
              {{ cancelText || 'Cancel' }}
            </button>
            <button 
              class="btn" 
              :class="danger ? 'btn-danger' : 'btn-primary'" 
              @click="emit('confirm')"
            >
              {{ confirmText || 'Confirm' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1001;
}

.confirm-dialog {
  width: 90%;
  max-width: 380px;
  text-align: center;
  animation: popIn 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

@keyframes popIn {
  from {
    opacity: 0;
    transform: scale(0.8);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.dialog-icon {
  width: 60px;
  height: 60px;
  margin: 0 auto 15px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(168, 85, 247, 0.2);
  color: #a855f7;
}

.dialog-icon.danger {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

h3 {
  margin: 0 0 10px 0;
  font-size: 1.2em;
  color: white;
}

p {
  margin: 0 0 20px 0;
  color: rgba(255, 255, 255, 0.7);
  font-size: 0.95em;
  line-height: 1.5;
}

.dialog-actions {
  display: flex;
  gap: 10px;
  justify-content: center;
}

.btn {
  padding: 10px 24px;
  border-radius: 999px;
  font-size: 0.9em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.btn:active {
  transform: scale(0.95);
}

.btn-primary {
  background: #a855f7;
  color: white;
}

.btn-primary:hover {
  background: #9333ea;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.15);
}

.btn-danger {
  background: #ef4444;
  color: white;
}

.btn-danger:hover {
  background: #dc2626;
}

.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
