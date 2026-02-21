import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import { User, Mail, Lock, ArrowRight, Loader2, Languages, ArrowLeft } from "lucide-react";
import { useTranslation } from "react-i18next";

interface UserInfo {
    id: number;
    username: string;
    email: string;
}

interface LoginProps {
    onLoginSuccess: (user: UserInfo) => void;
}

type Mode = "login" | "register" | "forgot";

export default function Login({ onLoginSuccess }: LoginProps) {
    const { t, i18n } = useTranslation();
    const [mode, setMode] = useState<Mode>("login");
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [email, setEmail] = useState("");
    const [newPassword, setNewPassword] = useState("");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<string | null>(null);
    const [rememberMe, setRememberMe] = useState(false);

    useEffect(() => {
        const savedUsername = localStorage.getItem("vp_username");
        const savedPassword = localStorage.getItem("vp_password");
        if (savedUsername && savedPassword) {
            setUsername(savedUsername);
            setPassword(savedPassword);
            setRememberMe(true);
        }
    }, []);

    const toggleLanguage = () => {
        const newLang = i18n.language.startsWith("zh") ? "en" : "zh";
        i18n.changeLanguage(newLang);
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);
        setSuccess(null);

        if (mode === "login") {
            if (!username.trim() || !password.trim()) {
                setError(t("fields_required"));
                return;
            }
            setLoading(true);
            try {
                const user = await invoke<UserInfo>("login", { username, password });
                if (rememberMe) {
                    localStorage.setItem("vp_username", username);
                    localStorage.setItem("vp_password", password);
                } else {
                    localStorage.removeItem("vp_username");
                    localStorage.removeItem("vp_password");
                }
                onLoginSuccess(user);
            } catch (err: any) {
                const errStr = String(err);
                if (errStr.includes("Invalid")) {
                    setError(t("login_error"));
                } else {
                    setError(t("login_error"));
                }
            } finally {
                setLoading(false);
            }
        } else if (mode === "register") {
            if (!username.trim() || !password.trim() || !email.trim()) {
                setError(t("fields_required"));
                return;
            }
            setLoading(true);
            try {
                const user = await invoke<UserInfo>("register", { username, password, email });
                onLoginSuccess(user);
            } catch (err: any) {
                const errStr = String(err);
                if (errStr.includes("already exists")) {
                    setError(t("username_exists"));
                } else {
                    setError(t("register_error"));
                }
            } finally {
                setLoading(false);
            }
        } else if (mode === "forgot") {
            if (!username.trim() || !email.trim() || !newPassword.trim()) {
                setError(t("fields_required"));
                return;
            }
            setLoading(true);
            try {
                await invoke("reset_password", { username, email, newPassword });
                setMode("login");
                setSuccess(t("reset_success"));
                // Clear fields
                setNewPassword("");
                setPassword("");
            } catch (err: any) {
                const errStr = String(err);
                if (errStr.includes("not found") || errStr.includes("mismatch")) {
                    setError(t("reset_error"));
                } else {
                    setError(t("reset_error"));
                }
            } finally {
                setLoading(false);
            }
        }
    };

    const switchMode = (newMode: Mode) => {
        setMode(newMode);
        setError(null);
        setSuccess(null);
    };

    const getTitle = () => {
        switch (mode) {
            case "login": return t("login");
            case "register": return t("register");
            case "forgot": return t("reset_password");
        }
    };

    const getButtonText = () => {
        switch (mode) {
            case "login": return t("login_btn");
            case "register": return t("register_btn");
            case "forgot": return t("reset_btn");
        }
    };

    return (
        <div className="min-h-screen bg-gradient-to-br from-blue-50 via-white to-indigo-50 flex items-center justify-center px-4">
            {/* Background decoration */}
            <div className="absolute inset-0 overflow-hidden pointer-events-none">
                <div className="absolute -top-40 -right-40 w-80 h-80 bg-blue-200 rounded-full opacity-20 blur-3xl" />
                <div className="absolute -bottom-40 -left-40 w-80 h-80 bg-indigo-200 rounded-full opacity-20 blur-3xl" />
            </div>

            {/* Language Switcher */}
            <div className="absolute top-4 right-4">
                <button
                    onClick={toggleLanguage}
                    className="bg-white p-2 rounded-lg shadow-sm hover:bg-gray-50 flex items-center gap-2 text-sm font-medium text-gray-600 transition-colors"
                >
                    <Languages size={18} />
                    <span>{i18n.language.startsWith("zh") ? "中文" : "English"}</span>
                </button>
            </div>

            <motion.div
                initial={{ opacity: 0, y: 30 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5 }}
                className="w-full max-w-md relative"
            >
                {/* Logo/Title */}
                <div className="text-center mb-8">
                    <motion.div
                        initial={{ scale: 0.8 }}
                        animate={{ scale: 1 }}
                        transition={{ delay: 0.1, type: "spring" }}
                        className="inline-flex items-center justify-center w-20 h-20 mb-4 bg-white rounded-3xl shadow-xl shadow-blue-100/50 p-2 overflow-hidden border border-gray-100"
                    >
                        <img src="/icon.png" alt="Logo" className="w-full h-full object-contain" />
                    </motion.div>
                    <h1 className="text-2xl font-bold text-gray-800">{t("app_title")}</h1>
                    <p className="text-gray-500 text-sm mt-1">{t("app_desc")}</p>
                </div>

                {/* Card */}
                <div className="bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100 p-8">
                    <AnimatePresence mode="wait">
                        <motion.div
                            key={mode}
                            initial={{ opacity: 0, x: 20 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -20 }}
                            transition={{ duration: 0.2 }}
                        >
                            {/* Back button for forgot password */}
                            {mode === "forgot" && (
                                <button
                                    onClick={() => switchMode("login")}
                                    className="flex items-center gap-1 text-sm text-gray-500 hover:text-blue-600 mb-4 transition-colors"
                                >
                                    <ArrowLeft size={16} />
                                    {t("back_to_login")}
                                </button>
                            )}

                            <h2 className="text-xl font-bold text-gray-800 mb-6">
                                {getTitle()}
                            </h2>

                            <form onSubmit={handleSubmit} className="space-y-4">
                                {/* Username */}
                                <div className="relative">
                                    <User size={18} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-gray-400" />
                                    <input
                                        type="text"
                                        placeholder={t("username")}
                                        value={username}
                                        onChange={(e) => setUsername(e.target.value)}
                                        className="w-full pl-11 pr-4 py-3 bg-gray-50 border border-gray-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all placeholder-gray-400 text-gray-800"
                                    />
                                </div>

                                {/* Email (register + forgot) */}
                                <AnimatePresence>
                                    {(mode === "register" || mode === "forgot") && (
                                        <motion.div
                                            initial={{ opacity: 0, height: 0 }}
                                            animate={{ opacity: 1, height: "auto" }}
                                            exit={{ opacity: 0, height: 0 }}
                                            className="relative overflow-hidden"
                                        >
                                            <Mail size={18} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-gray-400 z-10" />
                                            <input
                                                type="email"
                                                placeholder={t("email")}
                                                value={email}
                                                onChange={(e) => setEmail(e.target.value)}
                                                className="w-full pl-11 pr-4 py-3 bg-gray-50 border border-gray-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all placeholder-gray-400 text-gray-800"
                                            />
                                        </motion.div>
                                    )}
                                </AnimatePresence>

                                {/* Password (login + register) */}
                                {mode !== "forgot" && (
                                    <div className="space-y-4">
                                        <div className="relative">
                                            <Lock size={18} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-gray-400" />
                                            <input
                                                type="password"
                                                placeholder={t("password")}
                                                value={password}
                                                onChange={(e) => setPassword(e.target.value)}
                                                className="w-full pl-11 pr-4 py-3 bg-gray-50 border border-gray-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all placeholder-gray-400 text-gray-800"
                                            />
                                        </div>

                                        {/* Remember Me */}
                                        {mode === "login" && (
                                            <div className="flex items-center">
                                                <input
                                                    id="remember-me"
                                                    type="checkbox"
                                                    checked={rememberMe}
                                                    onChange={(e) => setRememberMe(e.target.checked)}
                                                    className="w-4 h-4 text-blue-600 bg-gray-50 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
                                                />
                                                <label htmlFor="remember-me" className="ml-2 text-sm text-gray-600 cursor-pointer select-none">
                                                    {t("remember_me")}
                                                </label>
                                            </div>
                                        )}
                                    </div>
                                )}

                                {/* New password (forgot mode) */}
                                {mode === "forgot" && (
                                    <div className="relative">
                                        <Lock size={18} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-gray-400" />
                                        <input
                                            type="password"
                                            placeholder={t("new_password")}
                                            value={newPassword}
                                            onChange={(e) => setNewPassword(e.target.value)}
                                            className="w-full pl-11 pr-4 py-3 bg-gray-50 border border-gray-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all placeholder-gray-400 text-gray-800"
                                        />
                                    </div>
                                )}

                                {/* Forgot password link (login only) */}
                                {mode === "login" && (
                                    <div className="text-right">
                                        <button
                                            type="button"
                                            onClick={() => switchMode("forgot")}
                                            className="text-sm text-blue-600 hover:text-blue-700 transition-colors"
                                        >
                                            {t("forgot_password")}
                                        </button>
                                    </div>
                                )}

                                {/* Error */}
                                <AnimatePresence>
                                    {error && (
                                        <motion.div
                                            initial={{ opacity: 0, y: -5 }}
                                            animate={{ opacity: 1, y: 0 }}
                                            exit={{ opacity: 0, y: -5 }}
                                            className="text-red-500 text-sm bg-red-50 px-4 py-2.5 rounded-lg border border-red-100"
                                        >
                                            {error}
                                        </motion.div>
                                    )}
                                </AnimatePresence>

                                {/* Success */}
                                <AnimatePresence>
                                    {success && (
                                        <motion.div
                                            initial={{ opacity: 0, y: -5 }}
                                            animate={{ opacity: 1, y: 0 }}
                                            exit={{ opacity: 0, y: -5 }}
                                            className="text-green-700 text-sm bg-green-50 px-4 py-2.5 rounded-lg border border-green-100"
                                        >
                                            {success}
                                        </motion.div>
                                    )}
                                </AnimatePresence>

                                {/* Submit */}
                                <button
                                    type="submit"
                                    disabled={loading}
                                    className="w-full bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-semibold py-3 px-6 rounded-xl shadow-md shadow-blue-200 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                                >
                                    {loading ? (
                                        <Loader2 className="animate-spin" size={20} />
                                    ) : (
                                        <>
                                            <span>{getButtonText()}</span>
                                            <ArrowRight size={18} />
                                        </>
                                    )}
                                </button>
                            </form>
                        </motion.div>
                    </AnimatePresence>

                    {/* Toggle login/register */}
                    {mode !== "forgot" && (
                        <div className="mt-6 text-center text-sm text-gray-500">
                            <span>{mode === "register" ? t("has_account") : t("no_account")}</span>
                            <button
                                onClick={() => switchMode(mode === "register" ? "login" : "register")}
                                className="ml-1 text-blue-600 hover:text-blue-700 font-semibold transition-colors"
                            >
                                {mode === "register" ? t("go_login") : t("go_register")}
                            </button>
                        </div>
                    )}
                </div>
            </motion.div>
        </div>
    );
}

export type { UserInfo };
