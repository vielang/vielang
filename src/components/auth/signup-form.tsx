"use client"

import * as React from "react"

import { MallSignUpForm } from "@/features/auth/components/mall-signup-form"

/**
 * Sign Up Form - Phone Registration
 *
 * Uses Vielang API phone-based registration with SMS verification.
 * Users will receive a verification code via SMS.
 */
export function SignUpForm() {
  return <MallSignUpForm />
}
