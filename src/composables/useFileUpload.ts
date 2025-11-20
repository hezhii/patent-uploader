import { ref, computed, readonly } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { httpClient } from '@/utils/httpClient';
import { useAppStore } from '@/stores';
import { useLoggerStore } from '@/stores/logger';
import type { UploadProgress, UploadResponse } from '@/types';

export function useFileUpload() {
  const loggerStore = useLoggerStore();
  const store = useAppStore();
  
  const uploading = ref(false);
  const uploadQueue = ref<File[]>([]);
  const uploadProgress = ref<UploadProgress[]>([]);
  const currentUploadIndex = ref(-1);
  const isPaused = ref(false);
  const onlyValidInvention = ref(false);

  const overallProgress = computed(() => {
    if (uploadProgress.value.length === 0) return 0;
    
    const totalProgress = uploadProgress.value.reduce((sum, item) => {
      return sum + (item.status === 'completed' ? 100 : item.progress);
    }, 0);
    
    return Math.round(totalProgress / uploadProgress.value.length);
  });

  const completedCount = computed(() => {
    return uploadProgress.value.filter(item => item.status === 'completed').length;
  });

  const failedCount = computed(() => {
    return uploadProgress.value.filter(item => item.status === 'failed').length;
  });

  function initializeUpload(files: File[]) {
    uploadQueue.value = [...files];
    uploadProgress.value = files.map((file, index) => ({
      fileIndex: index,
      fileName: file.name,
      progress: 0,
      status: 'pending' as const,
    }));
    currentUploadIndex.value = -1;
    loggerStore.info(`初始化上传队列，共 ${files.length} 个文件`);
  }

  async function startUpload(): Promise<void> {
    if (uploadQueue.value.length === 0) {
      loggerStore.warn('上传队列为空');
      return;
    }
    
    // 先登录
    loggerStore.info('准备上传文件，首先进行身份验证...');
    try {
      const { serverUrl, username, password } = store.serverConfig;
      
      if (!serverUrl || !username || !password) {
        throw new Error('请先配置服务器连接信息');
      }
      
      httpClient.setBaseUrl(serverUrl);
      await httpClient.login(username, password);
      loggerStore.success('身份验证成功');
    } catch (error) {
      const errorMsg = `登录失败: ${error instanceof Error ? error.message : String(error)}`;
      loggerStore.error(errorMsg);
      throw new Error(errorMsg);
    }
    
    uploading.value = true;
    isPaused.value = false;
    currentUploadIndex.value = 0;
    loggerStore.info(`开始上传 ${uploadQueue.value.length} 个文件`);

    try {
      // 逐个上传文件
      for (let i = 0; i < uploadQueue.value.length; i++) {
        // 检查是否暂停
        while (isPaused.value) {
          await new Promise(resolve => setTimeout(resolve, 100));
        }
        
        currentUploadIndex.value = i;
        loggerStore.info(`正在上传文件 ${i + 1}/${uploadQueue.value.length}: ${uploadQueue.value[i].name}`);
        
        try {
          await uploadSingleFile(i);
        } catch (error) {
          // 单个文件失败不中断整个上传流程
          loggerStore.error(`文件 ${uploadQueue.value[i].name} 上传失败，继续下一个`);
        }
        
        // 避免服务器过载，每次上传后稍作暂停
        if (i < uploadQueue.value.length - 1) {
          await new Promise(resolve => setTimeout(resolve, 500));
        }
      }
      loggerStore.success(`所有文件上传完成！成功: ${completedCount.value}, 失败: ${failedCount.value}`);
    } catch (error) {
      loggerStore.error(`上传过程中发生错误: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    } finally {
      uploading.value = false;
      currentUploadIndex.value = -1;
    }
  }
  
  function pauseUpload() {
    isPaused.value = !isPaused.value;
    if (isPaused.value) {
      loggerStore.info('上传已暂停');
    } else {
      loggerStore.info('上传已继续');
    }
  }

  async function uploadSingleFile(index: number): Promise<void> {
    const file = uploadQueue.value[index];
    const progressItem = uploadProgress.value[index];

    progressItem.status = 'uploading';
    progressItem.progress = 0;
    progressItem.error = undefined;

    try {
      // 从 File 对象中获取文件路径
      const filePath = (file as any).filePath;
      
      if (!filePath) {
        throw new Error('无法获取文件路径，请使用 Tauri 的文件选择器');
      }

      // 获取 token（需要先登录）
      const token = httpClient.getToken();
      if (!token) {
        throw new Error('未登录，无法上传文件');
      }

      const { serverUrl } = store.serverConfig;
      
      loggerStore.info(`通过 Tauri 上传文件: ${filePath}`);
      
      // 调用 Tauri 命令上传文件
      const result = await invoke<UploadResponse>('upload_file', {
        filePath,
        serverUrl,
        token,
        onlyValidInvention: onlyValidInvention.value,
      });

      progressItem.status = 'completed';
      progressItem.progress = 100;
      progressItem.result = result;
      loggerStore.success(`文件上传成功: ${file.name}`);
      
    } catch (error) {
      progressItem.status = 'failed';
      const errorMsg = error instanceof Error ? error.message : '上传失败';
      progressItem.error = errorMsg;
      loggerStore.error(`文件上传失败 ${file.name}: ${errorMsg}`);
      throw error;
    }
  }

  async function retryUpload(index: number): Promise<void> {
    if (index < 0 || index >= uploadQueue.value.length) return;
    
    const progressItem = uploadProgress.value[index];
    if (progressItem.status !== 'failed') return;

    loggerStore.info(`重试上传文件: ${progressItem.fileName}`);
    try {
      await uploadSingleFile(index);
    } catch (error) {
      loggerStore.error(`重试上传文件 ${progressItem.fileName} 失败: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  function clearUploadHistory() {
    uploadQueue.value = [];
    uploadProgress.value = [];
    currentUploadIndex.value = -1;
  }

  function setOnlyValidInvention(value: boolean) {
    onlyValidInvention.value = value;
  }

  return {
    uploading: readonly(uploading),
    uploadProgress: readonly(uploadProgress),
    currentUploadIndex: readonly(currentUploadIndex),
    isPaused: readonly(isPaused),
    overallProgress,
    completedCount,
    failedCount,
    initializeUpload,
    startUpload,
    pauseUpload,
    retryUpload,
    clearUploadHistory,
    setOnlyValidInvention,
  };
}