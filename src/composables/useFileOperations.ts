import { ref, computed, readonly } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { ScanResult, ColumnMapping } from '@/types';

export function useFileOperations() {
  const sourcePath = ref('');
  const targetPath = ref('');
  const scanResult = ref<ScanResult | null>(null);
  const convertedFiles = ref<File[]>([]);
  const scanning = ref(false);
  const converting = ref(false);
  
  const canConvert = computed(() => 
    scanResult.value && scanResult.value.file_count > 0 && targetPath.value
  );
  
  const canUpload = computed(() => 
    convertedFiles.value.length > 0
  );
  
  async function selectSourceFolder() {
    try {
      const selected = await open({
        directory: true,
        title: '选择源文件夹'
      });
      if (selected) {
        sourcePath.value = selected as string;
      }
    } catch (error) {
      console.error('选择文件夹失败:', error);
      throw error;
    }
  }
  
  async function selectTargetFolder() {
    try {
      const selected = await open({
        directory: true,
        title: '选择目标文件夹'
      });
      if (selected) {
        targetPath.value = selected as string;
      }
    } catch (error) {
      console.error('选择文件夹失败:', error);
      throw error;
    }
  }
  
  async function scanFiles() {
    if (!sourcePath.value) return;
    
    scanning.value = true;
    try {
      const result = await invoke<ScanResult>('scan_excel_files', {
        sourcePath: sourcePath.value
      });
      scanResult.value = result;
    } catch (error) {
      console.error('扫描文件失败:', error);
      throw error;
    } finally {
      scanning.value = false;
    }
  }
  
  async function startConversion(mappings: ColumnMapping[]) {
    if (!canConvert.value) return;
    
    converting.value = true;
    try {
      const convertedFilePaths = await invoke<string[]>('convert_excel_files', {
        sourcePath: sourcePath.value,
        targetPath: targetPath.value,
        mappings
      });
      
      // 将文件路径转换为File对象
      const files = await Promise.all(
        convertedFilePaths.map(async (filePath) => {
          // 在 Tauri 环境中，我们需要使用特殊的方式来读取文件
          const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'unknown.xlsx';
          
          // 创建一个虚拟的 File 对象，实际文件读取在上传时进行
          const file = new File([], fileName, { 
            type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' 
          });
          
          // 添加文件路径属性，用于后续读取
          (file as any).filePath = filePath;
          
          return file;
        })
      );
      
      convertedFiles.value = files;
      return files;
    } catch (error) {
      console.error('转换文件失败:', error);
      throw error;
    } finally {
      converting.value = false;
    }
  }
  
  function clearConvertedFiles() {
    convertedFiles.value = [];
  }
  
  return {
    sourcePath,
    targetPath,
    scanResult,
    convertedFiles: readonly(convertedFiles),
    scanning: readonly(scanning),
    converting: readonly(converting),
    canConvert,
    canUpload,
    selectSourceFolder,
    selectTargetFolder,
    scanFiles,
    startConversion,
    clearConvertedFiles
  };
}