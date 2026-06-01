// @vitest-environment happy-dom

import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import LogViewer from './LogViewer.vue';
import { useLoggerStore } from '@/stores/logger';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
}));

describe('LogViewer', () => {
  it('removes rendered log entries after clicking clear logs', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);

    const wrapper = mount(LogViewer, {
      global: {
        plugins: [pinia],
      },
    });

    const loggerStore = useLoggerStore();
    loggerStore.info('用于验证清空的日志');
    await nextTick();

    expect(wrapper.text()).toContain('用于验证清空的日志');

    const clearButton = wrapper
      .findAll('button')
      .find(button => button.text() === '清空日志');

    expect(clearButton).toBeDefined();
    await clearButton!.trigger('click');
    await nextTick();

    expect(loggerStore.logs).toHaveLength(0);
    expect(wrapper.text()).toContain('暂无日志记录');
    expect(wrapper.text()).not.toContain('用于验证清空的日志');
  });
});
