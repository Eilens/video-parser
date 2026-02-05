import { useState, useEffect } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Loader2, Download, User, ImageIcon, Languages } from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { useTranslation } from "react-i18next";
import { PhotoProvider, PhotoView } from 'react-photo-view';
import 'react-photo-view/dist/react-photo-view.css';
import "./i18n";


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
}

function App() {
  const { t, i18n } = useTranslation();
  const [url, setUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<VideoParseInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Proxied images for platforms with hotlink protection (e.g., Weibo)
  const [proxiedImages, setProxiedImages] = useState<{ [key: number]: string }>({});
  const [proxiedAvatar, setProxiedAvatar] = useState<string | null>(null);
  // Cached video path for playback
  const [cachedVideo, setCachedVideo] = useState<string | null>(null);

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

    try {
      const res = await invoke<VideoParseInfo>("parse_video", { url: targetUrl });
      setResult(res);
      // Update input if called programmatically
      if (urlArg) setUrl(urlArg);
    } catch (err: any) {
      console.error(err);
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);

  const showToast = (message: string, type: 'success' | 'error' = 'success') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };


  const handleDownload = async (fileUrl: string, type: 'video' | 'image', index?: number) => {
    try {
      const ext = type === 'video' ? 'mp4' : 'jpeg';
      const timestamp = new Date().getTime();
      const defaultName = `douyin_${type}_${timestamp}${index !== undefined ? `_${index}` : ''}.${ext}`;

      const savePath = await save({
        defaultPath: defaultName,
        filters: [{
          name: type === 'video' ? 'Video' : 'Image',
          extensions: [ext]
        }]
      });

      if (!savePath) return;

      showToast(t('toast_downloading'), 'success');

      await invoke('download_file', { url: fileUrl, savePath });

      showToast(t('toast_saved'), 'success');
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

  return (
    <div className="min-h-screen bg-gray-100 text-gray-900 font-sans flex flex-col items-center py-10 px-4 relative">
      <div className="w-full max-w-3xl space-y-8">

        {/* Language Switcher */}
        <div className="absolute top-4 right-4 md:right-0 md:top-0 md:relative md:flex md:justify-end">
          <button
            onClick={toggleLanguage}
            className="bg-white p-2 rounded-lg shadow-sm hover:bg-gray-50 flex items-center gap-2 text-sm font-medium text-gray-600 transition-colors"
          >
            <Languages size={18} />
            {/* Show current language */}
            <span>{i18n.language.startsWith('zh') ? '中文' : 'English'}</span>
          </button>
        </div>

        {/* Header */}
        <div className="text-center space-y-3 mt-8 md:mt-0">
          <h1 className="text-3xl md:text-4xl font-extrabold text-blue-600 tracking-tight">
            {t('app_title')}
          </h1>
          <p className="text-gray-600 max-w-lg mx-auto">
            {t('app_desc')}
          </p>
        </div>

        {/* Search Input */}
        <div className="bg-white p-4 rounded-xl shadow-lg border border-gray-200 flex flex-col md:flex-row items-center gap-3">
          <div className="flex-1 w-full relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" size={20} />
            <input
              type="text"
              className="w-full pl-10 pr-4 py-3 bg-gray-50 border border-gray-300 rounded-lg outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all placeholder-gray-400 text-gray-800"
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
              className="bg-red-50 text-red-700 p-4 rounded-lg border border-red-200 text-center font-medium shadow-sm"
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
                className="bg-white p-5 rounded-xl shadow-md border border-gray-100 flex items-center gap-4"
              >
                <img
                  src={result.platform === 'weibo' && proxiedAvatar ? proxiedAvatar : result.author.avatar}
                  alt={result.author.name}
                  className="w-16 h-16 rounded-full object-cover border-2 border-gray-100"
                  referrerPolicy="no-referrer"
                />
                <div className="flex-1">
                  <h3 className="font-bold text-xl text-gray-800 flex items-center gap-2">
                    {result.author.name}
                    <User size={16} className="text-gray-400" />
                  </h3>
                  <p className="text-gray-500 text-sm font-mono">{t('uid')}: {result.author.uid}</p>
                </div>
              </div>

              {/* Media Content */}
              <div className="bg-white rounded-xl shadow-md border border-gray-100 overflow-hidden">
                <div className="p-5 border-b border-gray-100">
                  <p className="text-gray-800 text-lg leading-relaxed whitespace-pre-wrap">
                    {result.title}
                  </p>
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
                          src={cachedVideo || result.video_url}
                          className="w-full h-full object-contain cursor-pointer"
                          poster={result.cover_url}
                          onClick={(e) => e.currentTarget.requestFullscreen()}
                        />
                      </div>
                      <div className="flex justify-end">
                        <button
                          onClick={() => handleDownload(result.video_url, 'video')}
                          className="inline-flex items-center space-x-2 bg-blue-600 hover:bg-blue-700 text-white px-6 py-2.5 rounded-lg font-semibold transition-colors shadow-sm"
                        >
                          <Download size={18} />
                          <span>{t('download_video')}</span>
                        </button>
                      </div>
                    </div>
                  )}

                  {/* Image Gallery */}
                  {result.images.length > 0 && (
                    <div className="space-y-4">
                      <h3 className="font-semibold text-gray-700 flex items-center gap-2">
                        <ImageIcon size={20} />
                        <span>{t('gallery')} ({result.images.length})</span>
                      </h3>
                      <PhotoProvider>
                        <div className="grid grid-cols-2 sm:grid-cols-3 gap-4">
                          {result.images.map((img, idx) => (
                            <PhotoView key={idx} src={proxiedImages[idx] || img.url}>
                              <div className="relative group rounded-lg overflow-hidden aspect-[3/4] shadow-sm border border-gray-200 cursor-zoom-in">
                                <img
                                  src={proxiedImages[idx] || img.url}
                                  alt={`Gallery ${idx}`}
                                  className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                  referrerPolicy="no-referrer"
                                />
                                <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                                  <button
                                    onClick={(e) => {
                                      e.stopPropagation(); // Prevent opening zoom when clicking download
                                      handleDownload(img.url, 'image', idx);
                                    }}
                                    className="bg-white text-gray-900 py-2 px-4 rounded-full font-bold text-sm shadow-lg hover:bg-gray-50 transition-colors flex items-center gap-2"
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
                    <div className="space-y-4">
                      <h3 className="font-semibold text-gray-700">{t('cover_image')}</h3>
                      <img src={result.cover_url} className="rounded-lg w-full max-w-md shadow-md border border-gray-200" />
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
    </div>
  );
}

export default App;
