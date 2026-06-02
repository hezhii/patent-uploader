<template>
  <div>
    <form @submit.prevent="handleSubmit" class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">
          服务器地址
        </label>
        <input
          v-model="localConfig.serverUrl"
          type="url"
          placeholder="https://api.example.com"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
          required
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">
          用户名
        </label>
        <input
          v-model="localConfig.username"
          type="text"
          placeholder="用户名或邮箱"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
          required
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">
          密码
        </label>
        <div class="relative">
          <input
            v-model="localConfig.password"
            :type="showPassword ? 'text' : 'password'"
            placeholder="登录密码"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            required
          />
          <button
            type="button"
            @click="showPassword = !showPassword"
            class="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
          >
            {{ showPassword ? '👁️' : '👁️‍🗨️' }}
          </button>
        </div>
      </div>

      <div class="flex space-x-3">
        <button
          type="button"
          @click="testConnection"
          :disabled="testing || !isFormValid"
          class="btn-secondary flex-1"
          :class="{ 'opacity-50 cursor-not-allowed': testing || !isFormValid }"
        >
          {{ testing ? '测试中...' : '测试连接' }}
        </button>
        
        <button
          type="submit"
          :disabled="!isFormValid"
          class="btn-primary flex-1"
          :class="{ 'opacity-50 cursor-not-allowed': !isFormValid }"
        >
          保存配置
        </button>
      </div>
    </form>

    <!-- 连接状态显示 -->
    <div v-if="connectionStatus" class="mt-4 p-3 rounded-md" :class="statusClasses">
      <div class="flex items-center">
        <span class="mr-2">{{ statusIcon }}</span>
        <span class="text-sm">{{ connectionStatus.message }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { httpClient } from '@/utils/httpClient';
import { useLoggerStore } from '@/stores/logger';
import type { ServerConfig, ConnectionStatus } from '@/types';

interface Props {
  modelValue: ServerConfig;
}

interface Emits {
  (e: 'update:modelValue', value: ServerConfig): void;
  (e: 'save', value: ServerConfig): void;
  (e: 'test', success: boolean): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const loggerStore = useLoggerStore();
const localConfig = ref<ServerConfig>({ ...props.modelValue });
const showPassword = ref(false);
const testing = ref(false);
const connectionStatus = ref<ConnectionStatus | null>(null);

const isFormValid = computed(() => {
  return localConfig.value.serverUrl && 
         localConfig.value.username && 
         localConfig.value.password;
});

const statusClasses = computed(() => {
  if (!connectionStatus.value) return '';
  
  const baseClasses = 'border-l-4';
  switch (connectionStatus.value.type) {
    case 'success':
      return `${baseClasses} border-green-400 bg-green-50 text-green-800`;
    case 'error':
      return `${baseClasses} border-red-400 bg-red-50 text-red-800`;
    case 'warning':
      return `${baseClasses} border-yellow-400 bg-yellow-50 text-yellow-800`;
    default:
      return `${baseClasses} border-blue-400 bg-blue-50 text-blue-800`;
  }
});

const statusIcon = computed(() => {
  if (!connectionStatus.value) return '';
  
  switch (connectionStatus.value.type) {
    case 'success': return '✅';
    case 'error': return '❌';
    case 'warning': return '⚠️';
    default: return 'ℹ️';
  }
});

function isSameConfig(a: ServerConfig, b: ServerConfig) {
  return a.serverUrl === b.serverUrl &&
         a.username === b.username &&
         a.password === b.password;
}

// 监听外部数据变化
watch(() => props.modelValue, (newValue) => {
  if (isSameConfig(newValue, localConfig.value)) return;
  localConfig.value = { ...newValue };
}, { deep: true });

// 监听本地数据变化，同步到外部
watch(localConfig, (newValue) => {
  if (isSameConfig(newValue, props.modelValue)) return;
  emit('update:modelValue', { ...newValue });
}, { deep: true });

async function testConnection() {
  if (!isFormValid.value) return;
  
  testing.value = true;
  connectionStatus.value = null;
  loggerStore.info('开始测试服务器连接...');
  
  try {
    httpClient.setBaseUrl(localConfig.value.serverUrl);
    const success = await httpClient.testConnection();
    
    if (success) {
      connectionStatus.value = {
        type: 'success',
        message: '服务器连接正常'
      };
      loggerStore.success(`服务器连接测试成功: ${localConfig.value.serverUrl}`);
      emit('test', true);
    } else {
      connectionStatus.value = {
        type: 'error',
        message: '无法连接到服务器'
      };
      loggerStore.error(`服务器连接测试失败: ${localConfig.value.serverUrl}`);
      emit('test', false);
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : '连接测试失败';
    connectionStatus.value = {
      type: 'error',
      message: errorMsg
    };
    loggerStore.error(`服务器连接测试异常: ${errorMsg}`);
    emit('test', false);
  } finally {
    testing.value = false;
  }
}

function handleSubmit() {
  if (!isFormValid.value) return;
  
  loggerStore.info(`保存服务器配置: ${localConfig.value.serverUrl}, 用户: ${localConfig.value.username}`);
  
  // 保存配置到 store
  emit('save', { ...localConfig.value });
  
  connectionStatus.value = {
    type: 'success',
    message: '配置已保存'
  };
  
  loggerStore.success('服务器配置保存成功');
}
</script>
