import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { motion } from "framer-motion";
import { X, Trash2, FolderOpen, Play, Loader2, Download as DownloadIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

interface DownloadRecord {
    id: number;
    user_id: number;
    url: string;
    title: string;
    cover_url: string;
    file_path: string;
    status: string;
    total_size: number;
    downloaded_size: number;
    created_at: string;
}

interface DownloadProgressPayload {
    id: number;
    downloaded: number;
    total: number | null;
    status: string;
}

interface DownloadsProps {
    visible: boolean;
    onClose: () => void;
    userId: number;
}

export default function Downloads({ visible, onClose, userId }: DownloadsProps) {
    const { t } = useTranslation();
    const [downloads, setDownloads] = useState<DownloadRecord[]>([]);
    const [loading, setLoading] = useState(false);
    const [deletingId, setDeletingId] = useState<number | null>(null);

    // Poll for downloads on mount and when opening
    useEffect(() => {
        if (visible && userId) {
            fetchDownloads();
        }
    }, [visible, userId]);

    const fetchDownloads = async () => {
        try {
            setLoading(true);
            const res = await invoke<DownloadRecord[]>("get_downloads", { userId });
            setDownloads(res);
        } catch (err) {
            console.error("Failed to fetch downloads:", err);
        } finally {
            setLoading(false);
        }
    };

    // Listen to Tauri progress events
    useEffect(() => {
        const unlistenPromise = listen<DownloadProgressPayload>("download://progress", (event) => {
            const { id, downloaded, total, status } = event.payload;
            setDownloads((prev) => {
                // Find existing record or reload if new (though typically UI triggers fetch after starting)
                const idx = prev.findIndex((dl) => dl.id === id);
                if (idx >= 0) {
                    const updated = [...prev];
                    updated[idx] = {
                        ...updated[idx],
                        downloaded_size: downloaded,
                        total_size: total || updated[idx].total_size,
                        status: status
                    };
                    return updated;
                } else {
                    // A new download started that we don't have in state yet, just fetch all again
                    fetchDownloads();
                    return prev;
                }
            });
        });

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, [userId]); // Rebind if user changes

    const removeDownload = async (id: number, delete_file: boolean) => {
        try {
            await invoke("remove_download_record", { id, deleteFile: delete_file });
            setDownloads((prev) => prev.filter((dl) => dl.id !== id));
            setDeletingId(null);
        } catch (err) {
            console.error("Failed to remove download:", err);
        }
    };

    const openFile = async (path: string) => {
        try {
            await invoke("open_path", { path });
        } catch (err) {
            console.error("Failed to open path:", err);
        }
    };

    const revealFile = async (path: string) => {
        try {
            await invoke("reveal_path", { path });
        } catch (err) {
            console.error("Failed to reveal path:", err);
        }
    };

    const formatSize = (bytes: number) => {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const sizes = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    };

    if (!visible) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 pb-20 sm:pb-6">
            <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="absolute inset-0 bg-black/40 backdrop-blur-sm"
                onClick={onClose}
            />

            <motion.div
                initial={{ opacity: 0, y: 20, scale: 0.95 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: 20, scale: 0.95 }}
                transition={{ type: "spring", duration: 0.5, bounce: 0.3 }}
                className="bg-white rounded-2xl shadow-2xl w-full max-w-2xl h-[80vh] flex flex-col relative z-10 overflow-hidden"
            >
                {/* Header */}
                <div className="p-5 border-b border-gray-100 flex items-center justify-between bg-white shrink-0">
                    <h2 className="text-xl font-bold flex items-center gap-2 text-gray-800">
                        <DownloadIcon className="text-blue-500" size={24} />
                        {t('downloads')}
                    </h2>
                    <button
                        onClick={onClose}
                        className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-full transition-colors focus:outline-none"
                    >
                        <X size={20} />
                    </button>
                </div>

                {/* Content */}
                <div className="flex-1 overflow-y-auto p-4 bg-gray-50/50">
                    {loading && downloads.length === 0 ? (
                        <div className="flex flex-col items-center justify-center h-full text-gray-400">
                            <Loader2 className="animate-spin mb-3" size={32} />
                            <p>{t('loading_downloads') || 'Loading...'}</p>
                        </div>
                    ) : downloads.length === 0 ? (
                        <div className="flex flex-col items-center justify-center h-full text-gray-400">
                            <DownloadIcon size={48} className="mb-4 text-gray-300" />
                            <p className="text-lg font-medium">{t('no_downloads') || 'No download history yet'}</p>
                        </div>
                    ) : (
                        <div className="space-y-4">
                            {downloads.map((dl) => {
                                const isDownloading = dl.status === "downloading";
                                const isFailed = dl.status === "failed";
                                const isCompleted = dl.status === "completed";
                                const progress =
                                    dl.total_size > 0
                                        ? Math.round((dl.downloaded_size / dl.total_size) * 100)
                                        : 0;

                                return (
                                    <motion.div
                                        key={dl.id}
                                        layout
                                        initial={{ opacity: 0, scale: 0.95 }}
                                        animate={{ opacity: 1, scale: 1 }}
                                        exit={{ opacity: 0, scale: 0.95 }}
                                        className="bg-white p-4 rounded-xl border border-gray-100 shadow-sm flex flex-col sm:flex-row gap-4 relative group"
                                    >
                                        {/* Delete button (top right on mobile, hover on desktop right) */}
                                        <button
                                            onClick={() => setDeletingId(dl.id)}
                                            className="absolute top-2 right-2 sm:top-auto sm:right-4 sm:translate-y-6 sm:opacity-0 sm:group-hover:opacity-100 p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-all"
                                            title={t('remove_record') || 'Remove record'}
                                        >
                                            <Trash2 size={18} />
                                        </button>

                                        {/* Delete Confirmation Overlay */}
                                        {deletingId === dl.id && (
                                            <div className="absolute inset-0 z-20 bg-white/95 backdrop-blur-sm rounded-xl flex flex-col items-center justify-center p-4">
                                                <p className="font-semibold text-gray-800 mb-4">{t('delete_confirm_title') || 'Delete Download?'}</p>
                                                <div className="flex gap-3 w-full max-w-xs">
                                                    <button
                                                        onClick={() => removeDownload(dl.id, false)}
                                                        className="flex-1 bg-gray-100 hover:bg-gray-200 text-gray-700 py-2 rounded-lg text-sm font-medium transition-colors"
                                                    >
                                                        {t('delete_record_only') || 'Record only'}
                                                    </button>
                                                    <button
                                                        onClick={() => removeDownload(dl.id, true)}
                                                        className="flex-1 bg-red-500 hover:bg-red-600 text-white py-2 rounded-lg text-sm font-medium transition-colors"
                                                    >
                                                        {t('delete_record_and_file') || 'Record & File'}
                                                    </button>
                                                </div>
                                                <button
                                                    onClick={() => setDeletingId(null)}
                                                    className="absolute top-2 right-2 p-2 text-gray-400 hover:text-gray-600 rounded-full"
                                                >
                                                    <X size={16} />
                                                </button>
                                            </div>
                                        )}

                                        {/* Cover image */}
                                        <div className="w-full sm:w-28 h-32 sm:h-20 shrink-0 rounded-lg overflow-hidden bg-gray-100 border border-gray-200">
                                            {dl.cover_url ? (
                                                <img
                                                    src={dl.cover_url}
                                                    alt={dl.title}
                                                    className="w-full h-full object-cover"
                                                    referrerPolicy="no-referrer"
                                                />
                                            ) : (
                                                <div className="w-full h-full flex items-center justify-center text-gray-300">
                                                    <Play size={24} />
                                                </div>
                                            )}
                                        </div>

                                        {/* Info */}
                                        <div className="flex-1 min-w-0 pr-8">
                                            <h3 className="font-semibold text-gray-800 truncate mb-1" title={dl.title}>
                                                {dl.title || 'Untitled'}
                                            </h3>
                                            <div className="flex items-center gap-2 text-xs mb-3">
                                                <span className={`px-2 py-0.5 rounded-full font-medium ${isCompleted ? 'bg-green-100 text-green-700' :
                                                    isFailed ? 'bg-red-100 text-red-700' :
                                                        'bg-blue-100 text-blue-700'
                                                    }`}>
                                                    {t(isCompleted ? 'completed' : isFailed ? 'failed' : 'downloading')}
                                                </span>
                                                <span className="text-gray-500">
                                                    {new Date(dl.created_at).toLocaleString()}
                                                </span>
                                            </div>

                                            {/* Progress Bar & Size info */}
                                            {isDownloading && (
                                                <div className="space-y-1.5 mt-auto">
                                                    <div className="h-2 w-full bg-gray-100 rounded-full overflow-hidden">
                                                        <div
                                                            className="h-full bg-blue-500 transition-all duration-300 ease-out"
                                                            style={{ width: `${progress}%` }}
                                                        />
                                                    </div>
                                                    <div className="flex justify-between text-xs text-gray-500 font-medium">
                                                        <span>
                                                            {formatSize(dl.downloaded_size)} / {dl.total_size > 0 ? formatSize(dl.total_size) : '???'}
                                                        </span>
                                                        <span>{progress}%</span>
                                                    </div>
                                                </div>
                                            )}

                                            {/* Action buttons (only if completed) */}
                                            {isCompleted && (
                                                <div className="flex items-center gap-3 mt-3">
                                                    <button
                                                        onClick={() => openFile(dl.file_path)}
                                                        className="text-sm flex items-center gap-1.5 text-blue-600 hover:text-blue-700 font-medium bg-blue-50 px-3 py-1.5 rounded-lg transition-colors"
                                                    >
                                                        <Play size={14} />
                                                        {t('open_file')}
                                                    </button>
                                                    <button
                                                        onClick={() => revealFile(dl.file_path)}
                                                        className="text-sm flex items-center gap-1.5 text-gray-600 hover:text-gray-800 font-medium bg-gray-100 px-3 py-1.5 rounded-lg transition-colors"
                                                    >
                                                        <FolderOpen size={14} />
                                                        {t('open_folder')}
                                                    </button>
                                                </div>
                                            )}

                                            {/* Failure message */}
                                            {isFailed && (
                                                <div className="text-sm text-red-600 mt-2">
                                                    Something went wrong while downloading.
                                                </div>
                                            )}
                                        </div>
                                    </motion.div>
                                );
                            })}
                        </div>
                    )}
                </div>
            </motion.div>
        </div>
    );
}
