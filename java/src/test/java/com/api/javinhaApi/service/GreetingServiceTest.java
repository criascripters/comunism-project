package com.api.javinhaApi.service;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;
import org.mockito.Mockito;

import com.api.javinhaApi.dto.UserDto;
import com.api.javinhaApi.model.User;
import com.api.javinhaApi.repository.UserRepository;

import java.util.Optional;

class GreetingServiceTest {

    @Test
    void greetWithoutNameReturnsDefault() {
        UserRepository repo = Mockito.mock(UserRepository.class);
        GreetingService svc = new GreetingService(repo);
        assertEquals("Hello, World!", svc.greet(null));
    }

    @Test
    void saveUserCallsRepository() {
        UserRepository repo = Mockito.mock(UserRepository.class);
        GreetingService svc = new GreetingService(repo);
        User saved = new User();
        saved.setId(1L);
        saved.setName("A");
        saved.setEmail("a@example.com");
        Mockito.when(repo.save(Mockito.any(User.class))).thenReturn(saved);

        UserDto dto = new UserDto(null, "A", "a@example.com");
        UserDto out = svc.saveUser(dto);

        assertNotNull(out.getId());
        assertEquals("A", out.getName());
        Mockito.verify(repo).save(Mockito.any(User.class));
    }
}
