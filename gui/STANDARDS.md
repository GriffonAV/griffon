### **React Naming Conventions Cheat Sheet**

#### **Components**

- **PascalCase** for component names and filenames:

  ```jsx
  // Good
  function UserProfile() { ... }
  // File: UserProfile.jsx

  // Bad
  function userProfile() { ... }
  // File: userprofile.jsx
  ```

#### **Higher-Order Components (HOCs)**

- Prefix with `with`:
  ```jsx
  function withAuth(Component) { ... }
  ```

#### **Custom Hooks**

- Prefix with `use`:
  ```jsx
  function useFormInput(initialValue) { ... }
  ```

#### **Props**

- **Descriptive names**:
  ```jsx
  <UserProfile userId={123} isAdmin={true} />
  ```
- **Boolean props**: Use `is`, `has`, or `should`:
  ```jsx
  <Button isDisabled={true} />
  ```
- **Event handlers**: Prefix with `on`:
  ```jsx
  <Button onClick={handleClick} />
  ```

#### **State**

- **Descriptive names**:
  ```jsx
  const [isLoading, setIsLoading] = useState(false);
  ```
- **State updaters**: Use `setX` pattern:
  ```jsx
  const [count, setCount] = useState(0);
  ```

#### **Event Handlers**

- Prefix with `handle`:
  ```jsx
  function handleSubmit() { ... }
  ```

#### **CSS & Styling**

- **Styled Components**: Use descriptive names:
  ```jsx
  const ButtonContainer = styled.div`...`;
  ```
- **CSS Modules**: Use kebab-case:
  ```css
  /* styles.module.css */
  .button-primary { ... }
  ```

#### **Folders**

- Group by feature/component:
  ```
  src/
    components/
      Button/
        Button.jsx
        Button.module.css
  ```

#### **Constants & Enums**

- **Constants**: `SCREAMING_SNAKE_CASE`:
  ```js
  const API_BASE_URL = "https://api.example.com";
  ```
- **Enums**: PascalCase for objects, `SCREAMING_SNAKE_CASE` for properties:
  ```js
  const UserRoles = { ADMIN: "admin", EDITOR: "editor" };
  ```

#### **Functions**

- **Descriptive and context-specific**:
  ```js
  function fetchUserData() { ... }
  ```
