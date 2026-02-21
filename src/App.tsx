import { useState, useEffect } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Loader2, Download, User, ImageIcon, Languages, Star, LogOut, Copy, Clock, Cloud, Sun, Moon, Monitor, Settings, ChevronDown } from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { useTranslation } from "react-i18next";
import { PhotoProvider, PhotoView } from 'react-photo-view';
import 'react-photo-view/dist/react-photo-view.css';
import "./i18n";
import Favorites from "./Favorites";
import Login, { type UserInfo } from "./Login";
import Profile from "./Profile";
import Downloads from "./Downloads";


export interface VideoQuality {
  quality: string;
  video_url: string;
  size?: number | null;
}

interface VideoParseInfo {
  video_url: string;
  cover_url: string;
  title: string;
  author: {
    uid: string;
    name: string;
    avatar: string;
  };
  images: Array<{ url: string }>;
  platform: string;
  video_qualities?: VideoQuality[];
}

function App() {
  const { t, i18n } = useTranslation();

  // Auth state
  const [currentUser, setCurrentUser] = useState<UserInfo | null>(null);

  // Theme state
  const [theme, setTheme] = useState<'light' | 'dark' | 'system'>(() => {
    return (localStorage.getItem('theme') as 'light' | 'dark' | 'system') || 'system';
  });

  useEffect(() => {
    const root = document.documentElement;
    const matcher = window.matchMedia('(prefers-color-scheme: dark)');

    const applyTheme = () => {
      if (theme === 'dark' || (theme === 'system' && matcher.matches)) {
        root.classList.add('dark');
      } else {
        root.classList.remove('dark');
      }
    };

    applyTheme();
    localStorage.setItem('theme', theme);

    // Listen for system preference changes if in system mode
    if (theme === 'system') {
      matcher.addEventListener('change', applyTheme);
      return () => matcher.removeEventListener('change', applyTheme);
    }
  }, [theme]);

  const cycleTheme = () => {
    setTheme(prev => prev === 'system' ? 'light' : prev === 'light' ? 'dark' : 'system');
  };

  const [url, setUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<VideoParseInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Proxied images for platforms with hotlink protection (e.g., Weibo)
  const [proxiedImages, setProxiedImages] = useState<{ [key: number]: string }>({});
  const [proxiedAvatar, setProxiedAvatar] = useState<string | null>(null);
  // Cached video path for playback
  const [cachedVideo, setCachedVideo] = useState<string | null>(null);

  // Selected video quality
  const [selectedQualityUrl, setSelectedQualityUrl] = useState<string | null>(null);

  // Favorites state
  const [showFavorites, setShowFavorites] = useState(false);
  const [isFavorited, setIsFavorited] = useState(false);
  const [favRefreshKey, setFavRefreshKey] = useState(0);

  // Downloads state
  const [showDownloads, setShowDownloads] = useState(false);

  // Profile & Dropdown state
  const [showProfile, setShowProfile] = useState(false);
  const [showUserMenu, setShowUserMenu] = useState(false);
  const [showSettingsMenu, setShowSettingsMenu] = useState(false);

  // Time state
  const [currentTime, setCurrentTime] = useState(new Date());
  const [weatherInfo, setWeatherInfo] = useState<{ temp: string; weather: string; city: string } | null>(null);

  useEffect(() => {
    const timer = setInterval(() => setCurrentTime(new Date()), 1000);
    // Fetch weather data
    invoke<any>("get_weather").then(data => {
      console.log("Weather data returned from Rust:", data);
      if (data && data.weather) {
        setWeatherInfo({
          temp: data.weather.temperature,
          weather: data.weather.weather,
          city: data.position && data.position.city ? data.position.city : (data.city || '未知')
        });
      }
    }).catch(err => console.error("Failed to fetch weather:", err));
    return () => clearInterval(timer);
  }, []);

  // Update window title when language changes
  useEffect(() => {
    getCurrentWindow().setTitle(t('app_title'));
  }, [i18n.language, t]);

  // Proxy images for Weibo platform
  useEffect(() => {
    if (result && result.platform === 'weibo') {
      // Proxy avatar
      if (result.author.avatar) {
        invoke<string>('proxy_image', { url: result.author.avatar })
          .then(dataUrl => setProxiedAvatar(dataUrl))
          .catch(err => console.error('Failed to proxy avatar:', err));
      }

      // Cache video if exists
      if (result.video_url) {
        console.log('Caching video:', result.video_url);
        invoke<string>('cache_video', { url: result.video_url })
          .then(path => {
            console.log('Video cached to:', path);
            const assetUrl = convertFileSrc(path);
            console.log('Asset URL:', assetUrl);
            setCachedVideo(assetUrl);
          })
          .catch(err => console.error('Failed to cache video:', err));
      }

      // Proxy all images
      result.images.forEach((img, idx) => {
        invoke<string>('proxy_image', { url: img.url })
          .then(dataUrl => {
            setProxiedImages(prev => ({ ...prev, [idx]: dataUrl }));
          })
          .catch(err => console.error(`Failed to proxy image ${idx}:`, err));
      });
    } else {
      // Reset proxied data for other platforms
      setProxiedImages({});
      setProxiedAvatar(null);
      setCachedVideo(null);
    }
  }, [result]);

  const handleParse = async (urlArg?: string) => {
    const targetUrl = urlArg || url;
    if (!targetUrl) return;
    setLoading(true);
    setError(null);
    setResult(null);
    setProxiedImages({});
    setProxiedAvatar(null);
    setIsFavorited(false);

    try {
      const res = await invoke<VideoParseInfo>("parse_video", { url: targetUrl });
      // Pre-select best quality if available, otherwise fallback to default video_url
      if (res.video_qualities && res.video_qualities.length > 0) {
        setSelectedQualityUrl(res.video_qualities[0].video_url);
      } else {
        setSelectedQualityUrl(res.video_url);
      }

      setResult(res);
      // Update input if called programmatically
      if (urlArg) setUrl(targetUrl);
      // Check if this URL is already favorited
      if (currentUser) {
        const favorited = await invoke<boolean>("is_favorited", { userId: currentUser.id, url: targetUrl });
        setIsFavorited(favorited);
      }
    } catch (err: any) {
      console.error(err);
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleFavorite = async () => {
    if (!result || !currentUser) return;
    const targetUrl = url;
    try {
      if (isFavorited) {
        // Need to get favorites and find the matching one to remove
        const favs = await invoke<any[]>("get_favorites", { userId: currentUser.id, platform: null });
        const match = favs.find((f: any) => f.url === targetUrl);
        if (match) {
          await invoke("remove_favorite", { id: match.id });
          setIsFavorited(false);
          showToast(t('favorite_removed'), 'success');
          setFavRefreshKey((k) => k + 1);
        }
      } else {
        await invoke("add_favorite", {
          userId: currentUser.id,
          url: targetUrl,
          title: result.title || '',
          platform: result.platform || '',
          coverUrl: result.cover_url || '',
          authorName: result.author.name || '',
        });
        setIsFavorited(true);
        showToast(t('favorite_added'), 'success');
        setFavRefreshKey((k) => k + 1);
      }
    } catch (err) {
      console.error('Failed to toggle favorite:', err);
    }
  };

  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);

  const showToast = (message: string, type: 'success' | 'error' = 'success') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  const handleCopyUrl = async (urlToCopy: string) => {
    try {
      await navigator.clipboard.writeText(urlToCopy);
      showToast(t('copy_link_success'), 'success');
    } catch (err: any) {
      console.error('Failed to copy text: ', err);
      showToast(t('error_download', { error: err.toString() }), 'error');
    }
  };


  const handleDownload = async (fileUrl: string, type: 'video' | 'image' | 'audio', index?: number) => {
    try {
      // Auto-detect extension from URL for audio, fallback to defaults
      let ext = { video: 'mp4', image: 'jpeg', audio: 'mp3' }[type];
      if (type === 'audio' || type === 'video') {
        try {
          const urlPath = new URL(fileUrl).pathname;
          const urlExt = urlPath.split('.').pop()?.toLowerCase();
          if (urlExt && ['mp3', 'm4a', 'aac', 'wav', 'ogg', 'flac', 'mp4', 'mov', 'avi', 'mkv'].includes(urlExt)) {
            ext = urlExt;
          }
        } catch { /* use default ext */ }
      }
      const timestamp = new Date().getTime();
      let safeTitle = result?.title ? result.title.replace(/[\\/:*?"<>|\r\n]/g, '').trim() : '';
      if (safeTitle.length > 40) safeTitle = safeTitle.substring(0, 40);
      const prefix = safeTitle ? `${safeTitle}_` : `${result?.platform || 'media'}_`;
      const defaultName = `${prefix}${type}_${timestamp}${index !== undefined ? `_${index}` : ''}.${ext}`;

      const savePath = await save({
        defaultPath: defaultName,
        filters: [{
          name: type === 'video' ? 'Video' : type === 'audio' ? 'Audio' : 'Image',
          extensions: [ext]
        }]
      });

      if (!savePath) return;

      showToast(t('toast_downloading'), 'success');

      // Do not await, let it run in the background
      invoke('download_file', {
        userId: currentUser?.id || 0,
        url: fileUrl,
        savePath,
        title: result?.title || '',
        coverUrl: result?.cover_url || ''
      }).then(() => {
        showToast(t('toast_saved'), 'success');
      }).catch((err) => {
        showToast(t('error_download', { error: err }), 'error');
      });

      // Show the progress panel if not already open
      setShowDownloads(true);
    } catch (err: any) {
      console.error(err);
      showToast(t('error_download', { error: err }), 'error');
    }
  };

  const toggleLanguage = () => {
    // If current is Chinese (zh or zh-CN), switch to English
    const newLang = i18n.language.startsWith('zh') ? 'en' : 'zh';
    i18n.changeLanguage(newLang);
  };

  const handleLogout = () => {
    setCurrentUser(null);
    setResult(null);
    setUrl("");
    setError(null);
  };

  // Show login page if not authenticated
  if (!currentUser) {
    return <Login onLoginSuccess={(user) => setCurrentUser(user)} />;
  }

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900 text-gray-900 dark:text-gray-100 font-sans flex flex-col items-center py-10 px-4 relative transition-colors duration-200">
      <div className="w-full max-w-3xl space-y-8">

        {/* Top Bar: User info + Favorites + Language + Logout */}
        <div className="absolute top-4 right-4 md:right-0 md:top-0 md:relative flex flex-col items-end gap-2 w-full pr-4 md:pr-0 pointer-events-none">
          {/* Weather & Time */}
          <div className="flex items-center gap-3 text-sm font-medium text-gray-600 dark:text-gray-400 pointer-events-auto">
            {weatherInfo && (
              <div className="flex items-center gap-1" title={`${weatherInfo.city} ${weatherInfo.weather}`}>
                <Cloud size={16} className="text-gray-500 dark:text-gray-400" />
                <span className="hidden sm:inline">{weatherInfo.city} {weatherInfo.temp}°C {weatherInfo.weather}</span>
              </div>
            )}
            <div className="flex items-center gap-1" title="Current Time">
              <Clock size={16} className="text-gray-500 dark:text-gray-400" />
              <span className="font-mono hidden sm:inline">
                {`${currentTime.getFullYear()}-${(currentTime.getMonth() + 1).toString().padStart(2, '0')}-${currentTime.getDate().toString().padStart(2, '0')} ${currentTime.getHours().toString().padStart(2, '0')}:${currentTime.getMinutes().toString().padStart(2, '0')}:${currentTime.getSeconds().toString().padStart(2, '0')} ${['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六'][currentTime.getDay()]}`}
              </span>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex items-center justify-end gap-2 flex-wrap pointer-events-auto">

            <button
              onClick={() => setShowDownloads(true)}
              className="bg-white dark:bg-gray-800 p-2 rounded-lg shadow-sm hover:bg-green-50 dark:hover:bg-gray-700 flex items-center gap-2 text-sm font-medium text-gray-600 dark:text-gray-300 hover:text-green-600 dark:hover:text-green-400 transition-colors cursor-pointer whitespace-nowrap"
            >
              <Download size={18} />
              <span className="hidden sm:inline">{t('downloads')}</span>
            </button>
            <button
              onClick={() => setShowFavorites(true)}
              className="bg-white dark:bg-gray-800 p-2 rounded-lg shadow-sm hover:bg-amber-50 dark:hover:bg-gray-700 flex items-center gap-2 text-sm font-medium text-gray-600 dark:text-gray-300 hover:text-amber-600 dark:hover:text-amber-400 transition-colors cursor-pointer whitespace-nowrap"
            >
              <Star size={18} />
              <span className="hidden sm:inline">{t('favorites')}</span>
            </button>

            {/* Settings Dropdown */}
            <div className="relative">
              <button
                onClick={() => setShowSettingsMenu(!showSettingsMenu)}
                className="bg-white dark:bg-gray-800 p-2 rounded-lg shadow-sm hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-2 text-sm font-medium text-gray-600 dark:text-gray-300 transition-colors cursor-pointer whitespace-nowrap"
                title={t('settings') || 'Settings'}
              >
                <Settings size={18} />
                <span className="hidden sm:inline">{t('settings') || 'Settings'}</span>
              </button>

              <AnimatePresence>
                {showSettingsMenu && (
                  <>
                    <div className="fixed inset-0 z-40" onClick={() => setShowSettingsMenu(false)} />
                    <motion.div
                      initial={{ opacity: 0, scale: 0.95, y: -10 }}
                      animate={{ opacity: 1, scale: 1, y: 0 }}
                      exit={{ opacity: 0, scale: 0.95, y: -10 }}
                      transition={{ duration: 0.15 }}
                      className="absolute right-0 top-full mt-2 w-40 bg-white dark:bg-gray-800 rounded-xl shadow-xl border border-gray-100 dark:border-gray-700 overflow-hidden z-50 text-gray-700 dark:text-gray-300 pointer-events-auto"
                    >
                      <div className="p-2 space-y-1">
                        <button
                          onClick={() => { cycleTheme(); setShowSettingsMenu(false); }}
                          className="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-3 text-sm transition-colors cursor-pointer"
                        >
                          {theme === 'light' ? <Sun size={16} /> : theme === 'dark' ? <Moon size={16} /> : <Monitor size={16} />}
                          {theme === 'light' ? t('light_mode') || 'Light' : theme === 'dark' ? t('dark_mode') || 'Dark' : t('system_mode') || 'System'}
                        </button>
                        <button
                          onClick={() => { toggleLanguage(); setShowSettingsMenu(false); }}
                          className="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 flex items-center gap-3 text-sm transition-colors cursor-pointer"
                        >
                          <Languages size={16} />
                          {i18n.language.startsWith('zh') ? 'English' : '中文'}
                        </button>
                      </div>
                    </motion.div>
                  </>
                )}
              </AnimatePresence>
            </div>

            {/* User Dropdown */}
            <div className="relative">
              <button
                onClick={() => setShowUserMenu(!showUserMenu)}
                className="bg-white dark:bg-gray-800 p-2 rounded-lg shadow-sm hover:bg-blue-50 dark:hover:bg-gray-700 flex items-center gap-2 text-sm font-medium text-gray-600 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors cursor-pointer whitespace-nowrap"
              >
                <User size={16} />
                <span className="hidden sm:inline">{currentUser.username}</span>
                <ChevronDown size={14} className="ml-0.5 opacity-70 hidden sm:block" />
              </button>

              <AnimatePresence>
                {showUserMenu && (
                  <>
                    <div className="fixed inset-0 z-40" onClick={() => setShowUserMenu(false)} />
                    <motion.div
                      initial={{ opacity: 0, scale: 0.95, y: -10 }}
                      animate={{ opacity: 1, scale: 1, y: 0 }}
                      exit={{ opacity: 0, scale: 0.95, y: -10 }}
                      transition={{ duration: 0.15 }}
                      className="absolute right-0 top-full mt-2 w-40 bg-white dark:bg-gray-800 rounded-xl shadow-xl border border-gray-100 dark:border-gray-700 overflow-hidden z-50 text-gray-700 dark:text-gray-300 pointer-events-auto"
                    >
                      <div className="p-2 space-y-1">
                        <button
                          onClick={() => { setShowProfile(true); setShowUserMenu(false); }}
                          className="w-full text-left px-3 py-2 text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-gray-700 rounded-lg flex items-center gap-3 text-sm transition-colors cursor-pointer"
                        >
                          <User size={16} />
                          {t('edit_profile')}
                        </button>
                        <div className="h-px bg-gray-100 dark:bg-gray-700 my-1"></div>
                        <button
                          onClick={() => { handleLogout(); setShowUserMenu(false); }}
                          className="w-full text-left px-3 py-2 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-gray-700 rounded-lg flex items-center gap-3 text-sm transition-colors cursor-pointer"
                        >
                          <LogOut size={16} />
                          {t('logout')}
                        </button>
                      </div>
                    </motion.div>
                  </>
                )}
              </AnimatePresence>
            </div>
          </div>
        </div>

        {/* Header */}
        <div className="text-center space-y-3 mt-16 md:mt-0">
          <h1 className="text-3xl md:text-4xl font-extrabold text-blue-600 tracking-tight">
            {t('app_title')}
          </h1>
          <p className="text-gray-600 dark:text-gray-400 max-w-lg mx-auto relative z-10 drop-shadow-sm">
            {t('app_desc')}
          </p>
        </div>

        {/* Search Input */}
        <div className="bg-white dark:bg-gray-800 p-4 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 flex flex-col md:flex-row items-center gap-3 relative z-10 transition-colors duration-200">
          <div className="flex-1 w-full relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 dark:text-gray-500" size={20} />
            <input
              type="text"
              className="w-full pl-10 pr-4 py-3 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all placeholder-gray-400 dark:placeholder-gray-500 text-gray-800 dark:text-gray-100"
              placeholder={t('placeholder')}
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleParse()}
            />
          </div>
          <button
            onClick={() => handleParse()}
            disabled={loading || !url}
            className="w-full md:w-auto bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-semibold py-3 px-8 rounded-lg shadow-md transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2"
          >
            {loading ? (
              <>
                <Loader2 className="animate-spin" size={20} />
                <span>{t('parsing')}</span>
              </>
            ) : (
              <span>{t('parse_btn')}</span>
            )}
          </button>
        </div>

        {/* Results Area */}
        <AnimatePresence mode="wait">
          {error && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              exit={{ opacity: 0, height: 0 }}
              className="bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-400 p-4 rounded-lg border border-red-200 dark:border-red-800 text-center font-medium shadow-sm transition-colors"
            >
              {error}
            </motion.div>
          )}

          {result ? (
            <motion.div
              key="result-view"
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="space-y-6"
            >
              {/* Author Card */}
              <div
                className="bg-white dark:bg-gray-800 p-5 rounded-xl shadow-md border border-gray-100 dark:border-gray-700 flex items-center gap-4 transition-colors"
              >
                <img
                  src={result.platform === 'weibo' && proxiedAvatar ? proxiedAvatar : result.author.avatar}
                  alt={result.author.name}
                  className="w-16 h-16 rounded-full object-cover border-2 border-gray-100"
                  referrerPolicy="no-referrer"
                />
                <div className="flex-1">
                  <h3 className="font-bold text-xl text-gray-800 dark:text-gray-100 flex items-center gap-2">
                    {result.author.name}
                    <User size={16} className="text-gray-400" />
                  </h3>
                  <p className="text-gray-500 dark:text-gray-400 text-sm font-mono">{t('uid')}: {result.author.uid}</p>
                </div>
              </div>

              {/* Media Content */}
              <div className="bg-white dark:bg-gray-800 rounded-xl shadow-md border border-gray-100 dark:border-gray-700 overflow-hidden transition-colors">
                <div className="p-5 border-b border-gray-100 dark:border-gray-700 flex items-start justify-between gap-3 transition-colors">
                  <p className="text-gray-800 dark:text-gray-200 text-lg leading-relaxed whitespace-pre-wrap flex-1">
                    {result.title}
                  </p>
                  <button
                    onClick={handleToggleFavorite}
                    className={`flex-shrink-0 p-2.5 rounded-xl transition-all ${isFavorited
                      ? 'bg-amber-50 dark:bg-amber-900/40 text-amber-500 hover:bg-amber-100 dark:hover:bg-amber-900/60'
                      : 'bg-gray-50 dark:bg-gray-700 text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-600 hover:text-amber-500 dark:hover:text-amber-400'
                      }`}
                    title={isFavorited ? t('remove_favorite') : t('add_favorite')}
                  >
                    <Star size={22} className={isFavorited ? 'fill-amber-500' : ''} />
                  </button>
                </div>

                <div className="p-5 space-y-6">
                  {/* Video Player */}
                  {result.video_url && (
                    <div className="space-y-4">
                      <div className="bg-black rounded-lg overflow-hidden aspect-video shadow-sm flex items-center justify-center">
                        {/* @ts-ignore */}
                        <video
                          {...({ referrerPolicy: "no-referrer" } as any)}
                          controls
                          src={cachedVideo || selectedQualityUrl || result.video_url}
                          className="w-full h-full object-contain cursor-pointer"
                          poster={result.cover_url}
                          onClick={(e) => e.currentTarget.requestFullscreen()}
                        />
                      </div>
                      <div className="flex flex-col sm:flex-row justify-between items-center gap-4 bg-gray-50 dark:bg-gray-800/80 p-4 rounded-xl border border-gray-100 dark:border-gray-700 transition-colors">
                        <div className="flex items-center gap-3 w-full sm:w-auto">
                          {result.video_qualities && result.video_qualities.length > 0 ? (
                            <select
                              value={selectedQualityUrl || ''}
                              onChange={(e) => setSelectedQualityUrl(e.target.value)}
                              className="bg-gray-50 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 text-gray-900 dark:text-gray-100 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full sm:w-auto p-2.5 outline-none shadow-sm cursor-pointer transition-colors"
                            >
                              {result.video_qualities.map((q: VideoQuality, idx: number) => (
                                <option key={idx} value={q.video_url}>
                                  {q.quality} {q.size ? `(${((q.size) / (1024 * 1024)).toFixed(2)} MB)` : ''}
                                </option>
                              ))}
                            </select>
                          ) : (
                            <span className="text-gray-500 dark:text-gray-400 text-sm font-medium">{t('video_quality_default')}</span>
                          )}
                        </div>
                        <div className="flex items-center gap-2 w-full sm:w-auto">
                          <button
                            onClick={() => handleCopyUrl(selectedQualityUrl || result.video_url)}
                            className="w-full sm:w-auto inline-flex items-center justify-center space-x-2 bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-200 px-4 py-2.5 rounded-lg font-semibold transition-colors shadow-sm border border-gray-200 dark:border-gray-600"
                          >
                            <Copy size={18} />
                            <span>{t('copy_link')}</span>
                          </button>
                          <button
                            onClick={() => handleDownload(selectedQualityUrl || result.video_url, 'video')}
                            className="w-full sm:w-auto inline-flex items-center justify-center space-x-2 bg-blue-600 hover:bg-blue-700 text-white px-6 py-2.5 rounded-lg font-semibold transition-colors shadow-md cursor-pointer"
                          >
                            <Download size={18} />
                            <span>{t('download_video')}</span>
                          </button>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Image Gallery */}
                  {result.images.length > 0 && (
                    <div className="space-y-4 pt-4 border-t border-gray-100 dark:border-gray-700">
                      <h3 className="font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-2">
                        <ImageIcon size={20} />
                        <span>{t('gallery')} ({result.images.length})</span>
                      </h3>
                      <PhotoProvider>
                        <div className="grid grid-cols-2 sm:grid-cols-3 gap-4">
                          {result.images.map((img, idx) => (
                            <PhotoView key={idx} src={proxiedImages[idx] || img.url}>
                              <div className="relative group rounded-lg overflow-hidden aspect-[3/4] shadow-sm border border-gray-200 dark:border-gray-700 cursor-zoom-in">
                                <img
                                  src={proxiedImages[idx] || img.url}
                                  alt={`Gallery ${idx}`}
                                  className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                  referrerPolicy="no-referrer"
                                />
                                <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex flex-col items-center justify-center gap-3">
                                  <button
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      handleCopyUrl(img.url);
                                    }}
                                    className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 py-2 px-4 rounded-full font-bold text-sm shadow-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer"
                                  >
                                    <Copy size={14} />
                                    {t('copy_link')}
                                  </button>
                                  <button
                                    onClick={(e) => {
                                      e.stopPropagation(); // Prevent opening zoom when clicking download
                                      handleDownload(img.url, 'image', idx);
                                    }}
                                    className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 py-2 px-4 rounded-full font-bold text-sm shadow-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center gap-2 cursor-pointer"
                                  >
                                    <Download size={14} />
                                    {t('download_image')}
                                  </button>
                                </div>
                              </div>
                            </PhotoView>
                          ))}
                        </div>
                      </PhotoProvider>
                    </div>
                  )}

                  {/* Cover Image Fallback */}
                  {result.cover_url && !result.video_url && result.images.length === 0 && (
                    <div className="space-y-4 pt-4 border-t border-gray-100 dark:border-gray-700">
                      <h3 className="font-semibold text-gray-700 dark:text-gray-300">{t('cover_image')}</h3>
                      <img src={result.cover_url} className="rounded-lg w-full max-w-md shadow-md border border-gray-200 dark:border-gray-700" />
                    </div>
                  )}
                </div>
              </div>
            </motion.div>
          ) : null}
        </AnimatePresence>

        {/* Toast Notification */}
        <AnimatePresence>
          {toast && (
            <motion.div
              initial={{ opacity: 0, y: 50 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 50 }}
              className={`fixed bottom-10 left-1/2 transform -translate-x-1/2 px-6 py-3 rounded-full shadow-2xl font-medium text-white ${toast.type === 'error' ? 'bg-red-600' : 'bg-green-600'}`}
            >
              {toast.message}
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Favorites Panel */}
      <AnimatePresence>
        <Favorites
          visible={showFavorites}
          onClose={() => setShowFavorites(false)}
          onSelect={(selectedUrl) => {
            setUrl(selectedUrl);
            handleParse(selectedUrl);
          }}
          refreshKey={favRefreshKey}
          userId={currentUser.id}
        />
      </AnimatePresence>

      {/* Downloads Panel */}
      <AnimatePresence>
        <Downloads
          visible={showDownloads}
          onClose={() => setShowDownloads(false)}
          userId={currentUser.id}
        />
      </AnimatePresence>

      {/* Profile Modal */}
      <AnimatePresence>
        <Profile
          visible={showProfile}
          user={currentUser}
          onClose={() => setShowProfile(false)}
          onUpdated={(updatedUser) => setCurrentUser(updatedUser)}
        />
      </AnimatePresence>
    </div>
  );
}

export default App;
