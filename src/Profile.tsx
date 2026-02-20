import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import { X, User, Mail, Lock, Save, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";

interface UserInfo {
    id: number;
    username: string;
    email: string;
}

interface ProfileProps {
    visible: boolean;
    user: UserInfo;
    onClose: () => void;
    onUpdated: (user: UserInfo) => void;
}

export default function Profile({ visible, user, onClose, onUpdated }: ProfileProps) {
    const { t } = useTranslation();
    const [newUsername, setNewUsername] = useState("");
    const [newPassword, setNewPassword] = useState("");
    const [loading, setLoading] = useState(false);
    const [message, setMessage] = useState<{ text: string; type: "success" | "error" } | null>(null);

    const handleSave = async (e: React.FormEvent) => {
        e.preventDefault();
        setMessage(null);

        const hasUsername = newUsername.trim().length > 0;
        const hasPassword = newPassword.trim().length > 0;

        if (!hasUsername && !hasPassword) {
            setMessage({ text: t("no_changes"), type: "error" });
            return;
        }

        setLoading(true);
        try {
            const updated = await invoke<UserInfo>("update_profile", {
                id: user.id,
                newUsername: hasUsername ? newUsername.trim() : null,
                newPassword: hasPassword ? newPassword.trim() : null,
            });
            setMessage({ text: t("profile_updated"), type: "success" });
            setNewUsername("");
            setNewPassword("");
            onUpdated(updated);
        } catch (err: any) {
            const errStr = String(err);
            if (errStr.includes("already exists")) {
                setMessage({ text: t("username_exists"), type: "error" });
            } else {
                setMessage({ text: String(err), type: "error" });
            }
        } finally {
            setLoading(false);
        }
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
                className="bg-white rounded-2xl shadow-2xl w-[90vw] max-w-md overflow-hidden"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
                    <h2 className="text-xl font-bold text-gray-800">{t("edit_profile")}</h2>
                    <button
                        onClick={onClose}
                        className="p-2 rounded-lg hover:bg-gray-100 text-gray-500 transition-colors"
                    >
                        <X size={20} />
                    </button>
                </div>

                <form onSubmit={handleSave} className="p-6 space-y-5">
                    {/* Current email (read-only) */}
                    <div>
                        <label className="text-sm font-medium text-gray-500 mb-1.5 flex items-center gap-1.5">
                            <Mail size={14} />
                            {t("current_email")}
                        </label>
                        <div className="w-full px-4 py-3 bg-gray-50 border border-gray-200 rounded-xl text-gray-600">
                            {user.email}
                        </div>
                    </div>

                    {/* New username */}
                    <div>
                        <label className="text-sm font-medium text-gray-500 mb-1.5 flex items-center gap-1.5">
                            <User size={14} />
                            {t("new_username")}
                        </label>
                        <input
                            type="text"
                            placeholder={user.username}
                            value={newUsername}
                            onChange={(e) => setNewUsername(e.target.value)}
                            className="w-full px-4 py-3 bg-gray-50 border border-gray-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all placeholder-gray-400 text-gray-800"
                        />
                        <p className="text-xs text-gray-400 mt-1">{t("leave_empty_keep")}</p>
                    </div>

                    {/* New password */}
                    <div>
                        <label className="text-sm font-medium text-gray-500 mb-1.5 flex items-center gap-1.5">
                            <Lock size={14} />
                            {t("change_password")}
                        </label>
                        <input
                            type="password"
                            placeholder="••••••••"
                            value={newPassword}
                            onChange={(e) => setNewPassword(e.target.value)}
                            className="w-full px-4 py-3 bg-gray-50 border border-gray-200 rounded-xl outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all placeholder-gray-400 text-gray-800"
                        />
                        <p className="text-xs text-gray-400 mt-1">{t("leave_empty_keep")}</p>
                    </div>

                    {/* Message */}
                    <AnimatePresence>
                        {message && (
                            <motion.div
                                initial={{ opacity: 0, y: -5 }}
                                animate={{ opacity: 1, y: 0 }}
                                exit={{ opacity: 0, y: -5 }}
                                className={`text-sm px-4 py-2.5 rounded-lg border ${message.type === "success"
                                        ? "text-green-700 bg-green-50 border-green-100"
                                        : "text-red-500 bg-red-50 border-red-100"
                                    }`}
                            >
                                {message.text}
                            </motion.div>
                        )}
                    </AnimatePresence>

                    {/* Save button */}
                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white font-semibold py-3 px-6 rounded-xl shadow-md shadow-blue-200 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                    >
                        {loading ? (
                            <Loader2 className="animate-spin" size={20} />
                        ) : (
                            <>
                                <Save size={18} />
                                <span>{t("save_changes")}</span>
                            </>
                        )}
                    </button>
                </form>
            </motion.div>
        </motion.div>
    );
}
