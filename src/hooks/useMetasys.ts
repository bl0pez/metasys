import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Uma, QueuedCommand } from '../types';

export function useMetasys() {
  const [umas, setUmas] = useState<Uma[]>([]);
  const [queue, setQueue] = useState<QueuedCommand[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchState = useCallback(async () => {
    try {
      const [umasData, queueData] = await Promise.all([
        invoke<Uma[]>('get_umas'),
        invoke<QueuedCommand[]>('get_queue_status')
      ]);
      setUmas(umasData);
      setQueue(queueData);
    } catch (err) {
      console.error('Error fetching state from backend', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchState();
    const interval = setInterval(fetchState, 2000); // Polling UI cada 2s para ver la cola
    return () => clearInterval(interval);
  }, [fetchState]);

  const addUma = async (name: string) => {
    try {
      await invoke('add_uma', { name });
      await fetchState();
    } catch (err) {
      console.error('Error adding UMA', err);
    }
  };

  const addPoint = async (umaId: string, name: string, reference: string, controlType: 'toggle' | 'analog' | 'temperature') => {
    try {
      await invoke('add_point', { umaId, name, reference, controlType });
      await fetchState();
    } catch (err) {
      console.error('Error adding Point', err);
    }
  };

  const removeUma = async (id: string) => {
    try {
      await invoke('remove_uma', { id });
      await fetchState();
    } catch (err) {
      console.error('Error removing UMA', err);
    }
  };

  const removePoint = async (umaId: string, pointId: string) => {
    try {
      await invoke('remove_point', { umaId, pointId });
      await fetchState();
    } catch (err) {
      console.error('Error removing Point', err);
    }
  };

  const enqueueCommand = async (umaName: string, pointName: string, reference: string, action: string) => {
    try {
      await invoke('enqueue_command', { umaName, pointName, reference, action });
      await fetchState();
    } catch (err) {
      console.error('Error enqueueing command', err);
    }
  };

  return {
    umas,
    queue,
    loading,
    addUma,
    addPoint,
    removeUma,
    removePoint,
    enqueueCommand
  };
}
