// =====================================================================================
// File: src/services/payment.ts
// Description: Payment API service for fetching payment data from the backend microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

import { Payment } from "../components/PaymentList";

/**
 * Fetches the payment list from the backend payment microservice.
 * @returns Promise<Payment[]>
 */
export async function fetchPayments(endpoint = "http://localhost:8082/payments"): Promise<Payment[]> {
  const res = await fetch(endpoint);
  if (!res.ok) throw new Error("Failed to fetch payments");
  const data = await res.json();
  return data.payments as Payment[];
} 