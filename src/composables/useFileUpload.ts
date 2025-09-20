import { ref, computed, readonly } from 'vue';
import { httpClient } from '@/utils/httpClient';
import type { UploadProgress, UploadResponse } from '@/types';

export function useFileUpload() {
  const uploading = ref(false);
  const uploadQueue = ref<File[]>([]);
  const uploadProgress = ref<UploadProgress[]>([]);
  const currentUploadIndex = ref(-1);

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
  }

  async function startUpload(): Promise<void> {
    if (uploadQueue.value.length === 0) return;
    
    uploading.value = true;
    currentUploadIndex.value = 0;

    try {
      // 逐个上传文件
      for (let i = 0; i < uploadQueue.value.length; i++) {
        currentUploadIndex.value = i;
        await uploadSingleFile(i);
        
        // 避免服务器过载，每次上传后稍作暂停
        if (i < uploadQueue.value.length - 1) {
          await new Promise(resolve => setTimeout(resolve, 500));
        }
      }
    } finally {
      uploading.value = false;
      currentUploadIndex.value = -1;
    }
  }

  async function uploadSingleFile(index: number): Promise<void> {
    const file = uploadQueue.value[index];
    const progressItem = uploadProgress.value[index];

    progressItem.status = 'uploading';
    progressItem.progress = 0;
    progressItem.error = undefined;

    try {
      let actualFile = file;
      
      // 如果是从文件路径创建的虚拟File对象，需要读取实际文件
      if ((file as any).filePath) {
        const filePath = (file as any).filePath;
        // 在真实环境中，这里需要通过 Tauri 的文件系统 API 来读取文件
        // 目前先使用原始文件对象
        actualFile = file;
      }

      const result = await httpClient.uploadFile(actualFile, (progress) => {
        progressItem.progress = progress;
      });

      progressItem.status = 'completed';
      progressItem.progress = 100;
      progressItem.result = result;
      
    } catch (error) {
      progressItem.status = 'failed';
      progressItem.error = error instanceof Error ? error.message : '上传失败';
      throw error;
    }
  }

  async function retryUpload(index: number): Promise<void> {
    if (index < 0 || index >= uploadQueue.value.length) return;
    
    const progressItem = uploadProgress.value[index];
    if (progressItem.status !== 'failed') return;

    try {
      await uploadSingleFile(index);
    } catch (error) {
      console.error(`重试上传文件 ${progressItem.fileName} 失败:`, error);
    }
  }

  function clearUploadHistory() {
    uploadQueue.value = [];
    uploadProgress.value = [];
    currentUploadIndex.value = -1;
  }

  return {
    uploading: readonly(uploading),
    uploadProgress: readonly(uploadProgress),
    currentUploadIndex: readonly(currentUploadIndex),
    overallProgress,
    completedCount,
    failedCount,
    initializeUpload,
    startUpload,
    retryUpload,
    clearUploadHistory,
  };
}