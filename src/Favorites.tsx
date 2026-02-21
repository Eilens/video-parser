import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import { Star, Trash2, ExternalLink, X } from "lucide-react";
import { useTranslation } from "react-i18next";

interface Favorite {
  id: number;
  url: string;
  title: string;
  platform: string;
  cover_url: string;
  author_name: string;
  created_at: string;
}

const PLATFORMS = [
  { key: "all", i18nKey: "platform_all" },
  { key: "douyin", i18nKey: "platform_douyin" },
  { key: "xhs", i18nKey: "platform_xhs" },
  { key: "weibo", i18nKey: "platform_weibo" },
  { key: "bilibili", i18nKey: "platform_bilibili" },
  { key: "kuaishou", i18nKey: "platform_kuaishou" },
  { key: "pipixia", i18nKey: "platform_pipixia" },
  { key: "xigua", i18nKey: "platform_xigua" },
];

const PLATFORM_COLORS: Record<string, string> = {
  douyin: "#000000",
  xhs: "#FF2442",
  weibo: "#E6162D",
  bilibili: "#00A1D6",
  kuaishou: "#FF4906",
  pipixia: "#FF6699",
  xigua: "#FF6347",
};

interface FavoritesProps {
  visible: boolean;
  onClose: () => void;
  onSelect: (url: string) => void;
  refreshKey?: number;
  userId: number;
}

export default function Favorites({ visible, onClose, onSelect, refreshKey, userId }: FavoritesProps) {
  const { t } = useTranslation();
  const [favorites, setFavorites] = useState<Favorite[]>([]);
  const [activePlatform, setActivePlatform] = useState("all");
  const [loading, setLoading] = useState(false);

  const loadFavorites = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<Favorite[]>("get_favorites", {
        userId: userId,
        platform: activePlatform === "all" ? null : activePlatform,
      });
      setFavorites(data);
    } catch (err) {
      console.error("Failed to load favorites:", err);
    } finally {
      setLoading(false);
    }
  }, [activePlatform]);

  useEffect(() => {
    if (visible) {
      loadFavorites();
    }
  }, [visible, loadFavorites, refreshKey]);

  const handleRemove = async (id: number, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("remove_favorite", { id });
      setFavorites((prev) => prev.filter((f) => f.id !== id));
    } catch (err) {
      console.error("Failed to remove favorite:", err);
    }
  };

  const handleSelect = (url: string) => {
    onSelect(url);
    onClose();
  };

  const getPlatformColor = (platform: string) => {
    return PLATFORM_COLORS[platform] || "#6B7280";
  };

  if (!visible) return null;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
      onClick={onClose}
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 20 }}
        transition={{ type: "spring", duration: 0.4 }}
        className="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl w-[90vw] max-w-2xl max-h-[80vh] flex flex-col overflow-hidden transition-colors"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100 dark:border-gray-700 transition-colors">
          <h2 className="text-xl font-bold text-gray-800 dark:text-gray-100 flex items-center gap-2 transition-colors">
            <Star size={22} className="text-amber-500 fill-amber-500" />
            {t("favorites_title")}
          </h2>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400 transition-colors cursor-pointer"
          >
            <X size={20} />
          </button>
        </div>

        {/* Platform Tabs */}
        <div className="px-4 py-3 border-b border-gray-100 dark:border-gray-700 overflow-x-auto transition-colors">
          <div className="flex gap-2 min-w-max">
            {PLATFORMS.map((p) => (
              <button
                key={p.key}
                onClick={() => setActivePlatform(p.key)}
                className={`px-4 py-1.5 rounded-full text-sm font-medium transition-all whitespace-nowrap cursor-pointer ${activePlatform === p.key
                  ? "bg-blue-600 text-white shadow-md"
                  : "bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
                  }`}
              >
                {t(p.i18nKey)}
              </button>
            ))}
          </div>
        </div>

        {/* Favorites List */}
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {loading ? (
            <div className="flex items-center justify-center py-16">
              <div className="w-8 h-8 border-4 border-blue-200 border-t-blue-600 rounded-full animate-spin" />
            </div>
          ) : favorites.length === 0 ? (
            <div className="text-center py-16 text-gray-400">
              <Star size={48} className="mx-auto mb-4 opacity-30" />
              <p className="text-lg font-medium">{t("no_favorites")}</p>
              <p className="text-sm mt-1">{t("no_favorites_hint")}</p>
            </div>
          ) : (
            <AnimatePresence>
              {favorites.map((fav) => (
                <motion.div
                  key={fav.id}
                  layout
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, x: -100 }}
                  onClick={() => handleSelect(fav.url)}
                  className="flex items-center gap-4 p-4 bg-gray-50 dark:bg-gray-800/50 hover:bg-blue-50 dark:hover:bg-gray-700 rounded-xl border border-gray-100 dark:border-gray-700 hover:border-blue-200 dark:hover:border-gray-600 cursor-pointer transition-all group"
                >
                  {/* Cover thumbnail */}
                  {fav.cover_url ? (
                    <img
                      src={fav.cover_url}
                      alt=""
                      className="w-16 h-16 rounded-lg object-cover flex-shrink-0 border border-gray-200"
                      referrerPolicy="no-referrer"
                    />
                  ) : (
                    <div className="w-16 h-16 rounded-lg bg-gray-200 dark:bg-gray-700 flex-shrink-0 flex items-center justify-center transition-colors">
                      <ExternalLink size={20} className="text-gray-400 dark:text-gray-500" />
                    </div>
                  )}

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-semibold text-gray-800 dark:text-gray-200 truncate transition-colors">
                      {fav.title || fav.url}
                    </p>
                    <div className="flex items-center gap-2 mt-1">
                      <span
                        className="inline-block px-2 py-0.5 rounded text-xs font-semibold text-white"
                        style={{ backgroundColor: getPlatformColor(fav.platform) }}
                      >
                        {fav.platform}
                      </span>
                      {fav.author_name && (
                        <span className="text-xs text-gray-500 dark:text-gray-400 truncate transition-colors">
                          {fav.author_name}
                        </span>
                      )}
                    </div>
                  </div>

                  {/* Delete button */}
                  <button
                    onClick={(e) => handleRemove(fav.id, e)}
                    className="p-2 rounded-lg text-gray-400 dark:text-gray-500 hover:text-red-500 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 transition-colors opacity-0 group-hover:opacity-100 flex-shrink-0 cursor-pointer"
                    title={t("remove_favorite")}
                  >
                    <Trash2 size={18} />
                  </button>
                </motion.div>
              ))}
            </AnimatePresence>
          )}
        </div>
      </motion.div>
    </motion.div>
  );
}
