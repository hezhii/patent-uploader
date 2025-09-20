import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { ServerConfig, ColumnMapping } from '@/types';

export const useAppStore = defineStore('app', () => {
  // 服务器配置
  const serverConfig = ref<ServerConfig>({
    serverUrl: '',
    username: '',
    password: ''
  });

  // 列名映射配置
  const columnMappings = ref<ColumnMapping[]>([]);

  // 应用设置
  const settings = ref({
    theme: 'light' as 'light' | 'dark',
    autoSave: true,
    logLevel: 'info' as 'debug' | 'info' | 'warn' | 'error'
  });

  // 保存服务器配置
  function saveServerConfig(config: ServerConfig) {
    serverConfig.value = { ...config };
    if (settings.value.autoSave) {
      localStorage.setItem('patent-upload-server-config', JSON.stringify(config));
    }
  }

  // 加载服务器配置
  function loadServerConfig() {
    try {
      const saved = localStorage.getItem('patent-upload-server-config');
      if (saved) {
        serverConfig.value = JSON.parse(saved);
      }
    } catch (error) {
      console.error('加载服务器配置失败:', error);
    }
  }

  // 保存列名映射
  function saveColumnMappings(mappings: ColumnMapping[]) {
    columnMappings.value = [...mappings];
    if (settings.value.autoSave) {
      localStorage.setItem('patent-upload-column-mappings', JSON.stringify(mappings));
    }
  }

  // 加载列名映射
  function loadColumnMappings() {
    try {
      const saved = localStorage.getItem('patent-upload-column-mappings');
      if (saved) {
        columnMappings.value = JSON.parse(saved);
      }
    } catch (error) {
      console.error('加载列名映射失败:', error);
    }
  }

  // 添加列名映射
  function addColumnMapping(mapping: ColumnMapping) {
    columnMappings.value.push(mapping);
    if (settings.value.autoSave) {
      saveColumnMappings(columnMappings.value);
    }
  }

  // 删除列名映射
  function removeColumnMapping(index: number) {
    columnMappings.value.splice(index, 1);
    if (settings.value.autoSave) {
      saveColumnMappings(columnMappings.value);
    }
  }

  // 更新设置
  function updateSettings(newSettings: Partial<typeof settings.value>) {
    settings.value = { ...settings.value, ...newSettings };
    localStorage.setItem('patent-upload-settings', JSON.stringify(settings.value));
  }

  // 加载设置
  function loadSettings() {
    try {
      const saved = localStorage.getItem('patent-upload-settings');
      if (saved) {
        settings.value = { ...settings.value, ...JSON.parse(saved) };
      }
    } catch (error) {
      console.error('加载设置失败:', error);
    }
  }

  // 清除所有数据
  function clearAllData() {
    serverConfig.value = { serverUrl: '', username: '', password: '' };
    columnMappings.value = [];
    localStorage.removeItem('patent-upload-server-config');
    localStorage.removeItem('patent-upload-column-mappings');
  }

  // 初始化时加载数据
  loadServerConfig();
  loadColumnMappings();
  loadSettings();

  return {
    serverConfig,
    columnMappings,
    settings,
    saveServerConfig,
    loadServerConfig,
    saveColumnMappings,
    loadColumnMappings,
    addColumnMapping,
    removeColumnMapping,
    updateSettings,
    loadSettings,
    clearAllData,
  };
});