---
name: checkout
description: "PayPal JavaScript SDK for integrating checkout, payments, subscriptions, and orders into web applications."
metadata:
  languages: "javascript"
  versions: "1.0.3"
  updated-on: "2026-03-01"
  source: maintainer
  tags: "paypal,payments,checkout,subscriptions,orders"
---

# PayPal JavaScript SDK Coding Guide

## 1. Golden Rule

**Always use the official PayPal JavaScript SDK packages:**
- Core SDK: `@paypal/paypal-js` 
- React wrapper: `@paypal/react-paypal-js` 

**Never use deprecated or unofficial libraries.** These are the only supported PayPal JS SDK packages maintained by PayPal. 

## 2. Installation

### npm
```bash
npm install @paypal/paypal-js
# For React applications
npm install @paypal/react-paypal-js
```

### yarn
```bash
yarn add @paypal/paypal-js
# For React applications  
yarn add @paypal/react-paypal-js
```

### pnpm
```bash
pnpm add @paypal/paypal-js
# For React applications
pnpm add @paypal/react-paypal-js
``` 

**Environment Variables (Optional):**
```bash
PAYPAL_CLIENT_ID=your_client_id_here
PAYPAL_ENVIRONMENT=sandbox # or production
```

## 3. Initialization

### Vanilla JavaScript
```javascript
import { loadScript } from "@paypal/paypal-js";

// Basic initialization
const paypal = await loadScript({
    clientId: "your-client-id",
    currency: "USD"
});
``` 

### React Application
```javascript
import { PayPalScriptProvider } from "@paypal/react-paypal-js";

function App() {
    const initialOptions = {
        clientId: "your-client-id",
        currency: "USD",
        intent: "capture"
    };

    return (
        <PayPalScriptProvider options={initialOptions}>
            {/* Your PayPal components here */}
        </PayPalScriptProvider>
    );
}
``` 

The `PayPalScriptProvider` manages script loading and provides context to child components using React's Context API. 

## 4. Core API Surfaces

### PayPal Buttons

**Minimal Example:**
```javascript
import { PayPalButtons } from "@paypal/react-paypal-js";

<PayPalButtons
    createOrder={(data, actions) => {
        return actions.order.create({
            purchase_units: [{
                amount: { value: "10.00" }
            }]
        });
    }}
    onApprove={(data, actions) => {
        return actions.order.capture();
    }}
/>
```

**Advanced Example:**
```javascript
<PayPalButtons
    style={{
        layout: "vertical",
        color: "gold",
        shape: "rect",
        label: "paypal"
    }}
    createOrder={createOrder}
    onApprove={onApprove}
    onCancel={onCancel}
    onError={onError}
    fundingSource="paypal"
    disabled={false}
    forceReRender={[amount, currency]}
/>
``` 

### Braintree Integration

**Minimal Example:**
```javascript
import { BraintreePayPalButtons } from "@paypal/react-paypal-js";

<PayPalScriptProvider options={{
    clientId: "your-client-id",
    dataClientToken: "your-braintree-client-token"
}}>
    <BraintreePayPalButtons
        createOrder={(data, actions) => {
            return actions.braintree.createPayment({
                flow: "checkout",
                amount: "10.00",
                currency: "USD"
            });
        }}
        onApprove={(data, actions) => {
            return actions.braintree.tokenizePayment(data);
        }}
    />
</PayPalScriptProvider>
``` 

### Messages and Marks
```javascript
import { PayPalMessages, PayPalMarks } from "@paypal/react-paypal-js";

<PayPalMessages
    amount="10.00"
    placement="home"
    style={{
        layout: "text",
        logo: { type: "primary" }
    }}
/>

<PayPalMarks />
``` 

## 5. Advanced Features

### Error Handling

The SDK provides comprehensive error handling through callback functions and component lifecycle management:

```javascript
const onError = (err) => {
    console.error("PayPal error:", err);
    // Handle specific error types
    if (err.name === "VALIDATION_ERROR") {
        // Handle validation errors
    }
};

<PayPalButtons
    onError={onError}
    onCancel={onCancel}
    // other props
/>
```

The PayPal Buttons component includes built-in error handling that catches rendering failures and SDK initialization errors. 

### Script Loading States

```javascript
import { usePayPalScriptReducer } from "@paypal/react-paypal-js";

function PayPalStatus() {
    const [{ isLoaded, isPending, isRejected }] = usePayPalScriptReducer();
    
    if (isPending) return <div>Loading PayPal SDK...</div>;
    if (isLoaded) return <div>SDK ready</div>;
    if (isRejected) return <div>Failed to load SDK</div>;
}
```

### Component Lifecycle Management

The PayPal components handle their lifecycle automatically, including cleanup when components unmount: 

### Dynamic Configuration

```javascript
import { DISPATCH_ACTION } from "@paypal/react-paypal-js";

const [{ options }, dispatch] = usePayPalScriptReducer();

// Update currency dynamically
dispatch({
    type: DISPATCH_ACTION.RESET_OPTIONS,
    value: {
        ...options,
        currency: "EUR"
    }
});
``` 

### Eligibility Checking

The SDK automatically checks component eligibility before rendering:

## 6. TypeScript Usage

### Import Types
```typescript
import type {
    PayPalScriptOptions,
    PayPalButtonsComponentOptions,
    PayPalNamespace,
    FUNDING_SOURCE
} from "@paypal/paypal-js";
```

### Type-Safe Component Usage
```typescript
import { PayPalButtons } from "@paypal/react-paypal-js";
import type { PayPalButtonsComponentOptions } from "@paypal/paypal-js";

const buttonOptions: PayPalButtonsComponentOptions = {
    style: {
        layout: "vertical",
        color: "gold"
    },
    createOrder: (data, actions) => {
        return actions.order.create({
            purchase_units: [{
                amount: { value: "10.00" }
            }]
        });
    },
    onApprove: (data, actions) => {
        return actions.order.capture();
    }
};

<PayPalButtons {...buttonOptions} />
```

### Script Options Interface
```typescript
const scriptOptions: PayPalScriptOptions = {
    clientId: "your-client-id",
    currency: "USD",
    components: ["buttons", "marks"],
    disableFunding: ["credit", "card"],
    dataClientToken: "braintree-token"
};
```


