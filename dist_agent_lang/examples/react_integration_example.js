// React Integration with dist_agent_lang
// Example showing how to integrate dist_agent_lang with React applications

import React, { useState, useEffect, useCallback } from 'react';
import { Runtime, Value } from 'dist_agent_lang';

// Custom hook for dist_agent_lang integration
function useDistAgentLang() {
  const [runtime, setRuntime] = useState(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Initialize dist_agent_lang runtime
    const initRuntime = async () => {
      try {
        const rt = new Runtime();
        await rt.initialize();
        setRuntime(rt);
        setIsLoading(false);
      } catch (error) {
        console.error('Failed to initialize dist_agent_lang:', error);
        setIsLoading(false);
      }
    };

    initRuntime();
  }, []);

  // Execute dist_agent_lang function
  const executeFunction = useCallback(async (functionName, params = {}) => {
    if (!runtime) {
      throw new Error('dist_agent_lang runtime not initialized');
    }

    try {
      // Convert JavaScript objects to dist_agent_lang values
      const distAgentParams = Object.entries(params).map(([key, value]) => ({
        key,
        value: jsToDistAgentValue(value)
      }));

      // Execute function
      const result = await runtime.callFunction(functionName, distAgentParams);

      // Convert result back to JavaScript
      return distAgentToJsValue(result);
    } catch (error) {
      console.error(`Error executing ${functionName}:`, error);
      throw error;
    }
  }, [runtime]);

  return { runtime, executeFunction, isLoading };
}

// Convert JavaScript values to dist_agent_lang values
function jsToDistAgentValue(jsValue) {
  if (jsValue === null || jsValue === undefined) {
    return Value.Null;
  }

  switch (typeof jsValue) {
    case 'string':
      return Value.String(jsValue);
    case 'number':
      return Number.isInteger(jsValue) ? Value.Int(jsValue) : Value.Float(jsValue);
    case 'boolean':
      return Value.Bool(jsValue);
    case 'object':
      if (Array.isArray(jsValue)) {
        return Value.Array(jsValue.map(jsToDistAgentValue));
      } else {
        const entries = Object.entries(jsValue).map(([key, value]) => ({
          key: Value.String(key),
          value: jsToDistAgentValue(value)
        }));
        return Value.Object(entries);
      }
    default:
      return Value.String(String(jsValue));
  }
}

// Convert dist_agent_lang values to JavaScript
function distAgentToJsValue(distAgentValue) {
  if (!distAgentValue) return null;

  switch (distAgentValue.type) {
    case 'String':
      return distAgentValue.value;
    case 'Int':
    case 'Float':
      return distAgentValue.value;
    case 'Bool':
      return distAgentValue.value;
    case 'Array':
      return distAgentValue.value.map(distAgentToJsValue);
    case 'Object':
      const obj = {};
      distAgentValue.value.forEach(({ key, value }) => {
        obj[key.value] = distAgentToJsValue(value);
      });
      return obj;
    case 'Null':
    default:
      return null;
  }
}

// React component using dist_agent_lang
function TodoApp() {
  const { executeFunction, isLoading } = useDistAgentLang();
  const [todos, setTodos] = useState([]);
  const [newTodo, setNewTodo] = useState('');
  const [filter, setFilter] = useState('all');
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Load todos on component mount
  useEffect(() => {
    if (!isLoading) {
      loadTodos();
    }
  }, [isLoading]);

  const loadTodos = async () => {
    try {
      const result = await executeFunction('getUserTodos', {
        userId: 'current_user',
        filter: filter
      });
      setTodos(result.todos || []);
    } catch (error) {
      console.error('Failed to load todos:', error);
    }
  };

  const addTodo = async (e) => {
    e.preventDefault();
    if (!newTodo.trim() || isSubmitting) return;

    setIsSubmitting(true);
    try {
      const result = await executeFunction('createTodo', {
        text: newTodo.trim(),
        userId: 'current_user',
        priority: 'medium'
      });

      setTodos(prevTodos => [...prevTodos, result.todo]);
      setNewTodo('');
    } catch (error) {
      console.error('Failed to add todo:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const toggleTodo = async (todoId) => {
    try {
      const result = await executeFunction('toggleTodo', {
        todoId: todoId,
        userId: 'current_user'
      });

      setTodos(prevTodos =>
        prevTodos.map(todo =>
          todo.id === todoId ? { ...todo, completed: result.completed } : todo
        )
      );
    } catch (error) {
      console.error('Failed to toggle todo:', error);
    }
  };

  const deleteTodo = async (todoId) => {
    try {
      await executeFunction('deleteTodo', {
        todoId: todoId,
        userId: 'current_user'
      });

      setTodos(prevTodos => prevTodos.filter(todo => todo.id !== todoId));
    } catch (error) {
      console.error('Failed to delete todo:', error);
    }
  };

  const filteredTodos = todos.filter(todo => {
    switch (filter) {
      case 'active':
        return !todo.completed;
      case 'completed':
        return todo.completed;
      default:
        return true;
    }
  });

  if (isLoading) {
    return (
      <div className="loading">
        <div className="spinner"></div>
        <p>Initializing dist_agent_lang runtime...</p>
      </div>
    );
  }

  return (
    <div className="todo-app">
      <header>
        <h1>🚀 Todo App</h1>
        <p>Powered by dist_agent_lang & React</p>
      </header>

      <form onSubmit={addTodo} className="todo-form">
        <input
          type="text"
          value={newTodo}
          onChange={(e) => setNewTodo(e.target.value)}
          placeholder="What needs to be done?"
          disabled={isSubmitting}
        />
        <button type="submit" disabled={isSubmitting || !newTodo.trim()}>
          {isSubmitting ? 'Adding...' : 'Add Todo'}
        </button>
      </form>

      <div className="filters">
        {['all', 'active', 'completed'].map(filterOption => (
          <button
            key={filterOption}
            className={filter === filterOption ? 'active' : ''}
            onClick={() => setFilter(filterOption)}
          >
            {filterOption.charAt(0).toUpperCase() + filterOption.slice(1)}
          </button>
        ))}
      </div>

      <div className="stats">
        <div className="stat">
          <span className="number">{todos.length}</span>
          <span className="label">Total</span>
        </div>
        <div className="stat">
          <span className="number">{todos.filter(t => !t.completed).length}</span>
          <span className="label">Active</span>
        </div>
        <div className="stat">
          <span className="number">{todos.filter(t => t.completed).length}</span>
          <span className="label">Completed</span>
        </div>
      </div>

      <div className="todo-list">
        {filteredTodos.length === 0 ? (
          <div className="empty-state">
            <p>No todos found</p>
          </div>
        ) : (
          filteredTodos.map(todo => (
            <div key={todo.id} className={`todo-item ${todo.completed ? 'completed' : ''}`}>
              <input
                type="checkbox"
                checked={todo.completed}
                onChange={() => toggleTodo(todo.id)}
                className="todo-checkbox"
              />
              <span className="todo-text">{todo.text}</span>
              <button
                onClick={() => deleteTodo(todo.id)}
                className="delete-btn"
              >
                ✕
              </button>
            </div>
          ))
        )}
      </div>

      <style jsx>{`
        .todo-app {
          max-width: 600px;
          margin: 0 auto;
          padding: 20px;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }

        header {
          text-align: center;
          margin-bottom: 30px;
        }

        header h1 {
          color: #3a7afe;
          margin-bottom: 10px;
        }

        .todo-form {
          display: flex;
          gap: 10px;
          margin-bottom: 20px;
        }

        .todo-form input {
          flex: 1;
          padding: 12px;
          border: 2px solid #e1e5e9;
          border-radius: 8px;
          font-size: 16px;
        }

        .todo-form input:focus {
          outline: none;
          border-color: #3a7afe;
        }

        .todo-form button {
          padding: 12px 24px;
          background: #3a7afe;
          color: white;
          border: none;
          border-radius: 8px;
          cursor: pointer;
          font-size: 16px;
        }

        .todo-form button:hover:not(:disabled) {
          background: #2356b8;
        }

        .todo-form button:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .filters {
          display: flex;
          gap: 10px;
          margin-bottom: 20px;
          flex-wrap: wrap;
        }

        .filters button {
          padding: 8px 16px;
          border: 2px solid #e1e5e9;
          background: white;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.3s ease;
        }

        .filters button.active {
          background: #3a7afe;
          color: white;
          border-color: #3a7afe;
        }

        .filters button:hover {
          border-color: #3a7afe;
        }

        .stats {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: 15px;
          margin-bottom: 20px;
        }

        .stat {
          background: #f8f9fa;
          padding: 15px;
          border-radius: 8px;
          text-align: center;
        }

        .stat .number {
          display: block;
          font-size: 24px;
          font-weight: bold;
          color: #3a7afe;
          margin-bottom: 5px;
        }

        .stat .label {
          font-size: 14px;
          color: #6c757d;
          text-transform: uppercase;
        }

        .todo-list {
          background: white;
          border-radius: 8px;
          box-shadow: 0 2px 10px rgba(0,0,0,0.1);
          overflow: hidden;
        }

        .todo-item {
          display: flex;
          align-items: center;
          padding: 16px 20px;
          border-bottom: 1px solid #e1e5e9;
          transition: background-color 0.3s ease;
        }

        .todo-item:last-child {
          border-bottom: none;
        }

        .todo-item:hover {
          background: #f8f9fa;
        }

        .todo-item.completed {
          opacity: 0.6;
        }

        .todo-item.completed .todo-text {
          text-decoration: line-through;
          color: #6c757d;
        }

        .todo-checkbox {
          width: 20px;
          height: 20px;
          margin-right: 15px;
          cursor: pointer;
        }

        .todo-text {
          flex: 1;
          font-size: 16px;
        }

        .delete-btn {
          padding: 6px 12px;
          background: #dc3545;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          transition: background-color 0.3s ease;
        }

        .delete-btn:hover {
          background: #c82333;
        }

        .empty-state {
          text-align: center;
          padding: 40px 20px;
          color: #6c757d;
        }

        .loading {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          min-height: 200px;
          color: #6c757d;
        }

        .spinner {
          width: 40px;
          height: 40px;
          border: 4px solid #f3f3f3;
          border-top: 4px solid #3a7afe;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin-bottom: 20px;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }

        @media (max-width: 600px) {
          .todo-app {
            padding: 10px;
          }

          .todo-form {
            flex-direction: column;
          }

          .filters {
            flex-direction: column;
          }

          .stats {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
}

// Higher-order component for dist_agent_lang integration
function withDistAgentLang(Component) {
  return function DistAgentLangWrapper(props) {
    const distAgentLang = useDistAgentLang();

    return <Component {...props} distAgentLang={distAgentLang} />;
  };
}

// Connected component example
const ConnectedTodoApp = withDistAgentLang(function({ distAgentLang, userId }) {
  // Component logic using distAgentLang
  const { executeFunction, isLoading } = distAgentLang;

  // ... component implementation
});

// Export the main component
export default TodoApp;
export { useDistAgentLang, withDistAgentLang };
