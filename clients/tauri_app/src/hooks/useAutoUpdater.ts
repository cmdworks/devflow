import { useState, useEffect, useCallback } from "react";
import { useDevFlowApi } from "./useDevFlowApi";
import type { UpdateCheckResponse } from "../types";

export function useAutoUpdater() {
  const api = useDevFlowApi();
  const [isUpdateModalOpen, setIsUpdateModalOpen] = useState<boolean>(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateCheckResponse | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState<boolean>(false);

  const handleCheckUpdates = useCallback(async () => {
    setIsCheckingUpdate(true);
    try {
      const info = await api.checkUpdate();
      setUpdateInfo(info);
      if (info.update_available) {
        setIsUpdateModalOpen(true);
      }
    } catch (e) {
      console.error("Failed to check for updates", e);
    } finally {
      setIsCheckingUpdate(false);
    }
  }, [api]);

  const handleInstallUpdate = useCallback(
    async (downloadUrl: string) => {
      return await api.installUpdate(downloadUrl);
    },
    [api]
  );

  const handleRestartApp = useCallback(async () => {
    await api.restartApp();
  }, [api]);

  // Silent background update check on app launch
  useEffect(() => {
    const timer = setTimeout(() => {
      api
        .checkUpdate()
        .then((info) => {
          setUpdateInfo(info);
        })
        .catch((e) => {
          console.debug("Background update check skipped:", e);
        });
    }, 2000);
    return () => clearTimeout(timer);
  }, [api]);

  return {
    isUpdateModalOpen,
    setIsUpdateModalOpen,
    updateInfo,
    setUpdateInfo,
    isCheckingUpdate,
    handleCheckUpdates,
    handleInstallUpdate,
    handleRestartApp,
  };
}
