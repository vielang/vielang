"use client";

import { useRouter } from "next/navigation";
import { useSession } from '@/features/auth/services/authClient';
import { Header } from "../../../components/common/header";
import { BasicUser } from '@/types';
import { UpgradeAccountModal } from "../../../components/docs/upgrade-account-modal";
import { useState } from "react";
import { Crown, CheckCircle, XCircle } from "lucide-react";

// Define an extended session type
type BasicSession = {
  user: BasicUser;
};

export function ProfilePageClient() {
  const router = useRouter();
  const { data, isPending } = useSession() as {
    data: BasicSession | null | undefined;
    isPending: boolean;
  };
  const [showUpgradeModal, setShowUpgradeModal] = useState(false);

  // Redirect to login if not authenticated
  if (!isPending && !data) {
    router.push("/auth/sign-in");
    return null;
  }

  if (isPending) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-50">
        <div className="text-center">
          <div className="mx-auto h-8 w-8 animate-spin rounded-full border-4 border-gray-300 border-t-blue-600"></div>
          <p className="mt-4 text-gray-600">Loading your profile...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen flex-col bg-gray-50">
      <Header />
      <main className="flex-1 py-12">
        <div className="container mx-auto max-w-4xl px-4">
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-gray-800">My Profile</h1>
            <p className="mt-2 text-gray-600">Manage your account information</p>
          </div>

          <div className="overflow-hidden rounded-xl bg-white shadow">
            <div className="flex flex-col md:flex-row">
              {/* Profile Image Section */}
              <div className="flex flex-col items-center justify-center bg-blue-50 p-8 md:w-1/3">
                {data?.user.image ? (
                  <img
                    src={data.user.image}
                    alt="Profile"
                    className="h-32 w-32 rounded-full object-cover ring-4 ring-white shadow-md"
                  />
                ) : (
                  <div className="flex h-32 w-32 items-center justify-center rounded-full bg-blue-100 text-3xl font-bold text-blue-500">
                    {data?.user?.username?.charAt(0).toUpperCase() || "?"}
                  </div>
                )}
                <h2 className="mt-4 text-xl font-semibold text-gray-800">
                  {data?.user?.username || "User"}
                </h2>
                <p className="text-sm text-gray-500">{data?.user?.email}</p>
              </div>

              {/* Profile Details Section */}
              <div className="flex-1 p-8">
                <h2 className="mb-6 text-xl font-semibold text-gray-800">
                  Account Information
                </h2>

                <div className="space-y-6">
                  <div>
                    <label className="text-xs font-medium uppercase tracking-wide text-gray-500">
                      Username
                    </label>
                    <p className="mt-1 text-gray-800">{data?.user?.username || "Not set"}</p>
                  </div>

                  <div>
                    <label className="text-xs font-medium uppercase tracking-wide text-gray-500">
                      Email Address
                    </label>
                    <p className="mt-1 text-gray-800">{data?.user?.email}</p>
                  </div>

                  <div>
                    <label className="text-xs font-medium uppercase tracking-wide text-gray-500">
                      Account Created
                    </label>
                    <p className="mt-1 text-gray-800">
                      {data?.user?.created
                        ? new Date(data.user.created).toLocaleDateString('en-US', {
                          year: 'numeric',
                          month: 'long',
                          day: 'numeric'
                        })
                        : "N/A"}
                    </p>
                  </div>

                  <div>
                    <label className="text-xs font-medium uppercase tracking-wide text-gray-500">
                      Last Updated
                    </label>
                    <p className="mt-1 text-gray-800">
                      {data?.user?.updated
                        ? new Date(data.user.updated).toLocaleDateString('en-US', {
                          year: 'numeric',
                          month: 'long',
                          day: 'numeric'
                        })
                        : "N/A"}
                    </p>
                  </div>

                  <div>
                    <label className="text-xs font-medium uppercase tracking-wide text-gray-500">
                      Account Status
                    </label>
                    <div className="mt-2">
                      {data?.user?.status === true ? (
                        <div className="inline-flex items-center gap-2 rounded-full bg-gradient-to-r from-yellow-100 to-orange-100 px-4 py-2 dark:from-yellow-900/30 dark:to-orange-900/30">
                          <Crown className="h-5 w-5 text-yellow-600 dark:text-yellow-400" />
                          <span className="font-semibold text-yellow-700 dark:text-yellow-300">
                            Premium Account
                          </span>
                          <CheckCircle className="h-4 w-4 text-green-600 dark:text-green-400" />
                        </div>
                      ) : (
                        <div className="space-y-3">
                          <div className="inline-flex items-center gap-2 rounded-full bg-gray-100 px-4 py-2 dark:bg-gray-800">
                            <XCircle className="h-4 w-4 text-gray-500" />
                            <span className="font-medium text-gray-600 dark:text-gray-400">
                              Free Account
                            </span>
                          </div>
                          <div>
                            <button
                              onClick={() => setShowUpgradeModal(true)}
                              className="inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-blue-600 to-indigo-600 px-4 py-2 text-sm font-medium text-white hover:from-blue-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-all"
                            >
                              <Crown className="h-4 w-4" />
                              Nâng cấp Premium - 200,000 VNĐ
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  </div>
                </div>

                {/* <div className="mt-8 flex justify-end">
                  <button className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2">
                    Edit Profile
                  </button>
                </div> */}
              </div>
            </div>
          </div>
        </div>
      </main>
      <UpgradeAccountModal
        isOpen={showUpgradeModal}
        onClose={() => setShowUpgradeModal(false)}
      />
      <footer className="bg-white py-6">
        <div className="container mx-auto max-w-4xl px-4 text-center text-sm text-gray-500">
          © {new Date().getFullYear()} VieLang. All rights reserved.
        </div>
      </footer>
    </div>
  );
}